package main

import (
	"fmt"
	"sort"
	"strings"
	"time"
)

// Constants are word-for-word those used in the Rust and JVM ports so the
// shared HASH_SEED / HASH_PRIME match across languages.
const (
	pickupDeadlineWindowHours = 48
	heavyParcelKg             = 5.0
	volumetricDivisor         = 5000.0
	checksumMod               = 97
	hashSeed                  = uint64(0xcbf29ce484222325)
	hashPrime                 = uint64(0x100000001b3)
)

// Push order matches Rust's compute_features exactly so the resulting
// features arrays are byte-identical.
func computeFeatures(p *ParcelResponse, now time.Time) []FeatureResponse {
	out := make([]FeatureResponse, 0, 10)
	if f := customsDocumentsRequired(p); f != nil {
		out = append(out, *f)
	}
	if f := heavyParcel(p); f != nil {
		out = append(out, *f)
	}
	if f := rateDelivery(p); f != nil {
		out = append(out, *f)
	}
	if f := pickupDeadlineSoon(p, now); f != nil {
		out = append(out, *f)
	}
	if f := rewardsAvailable(p, now); f != nil {
		out = append(out, *f)
	}
	if f := latestEventCause(p); f != nil {
		out = append(out, *f)
	}
	if f := greenTransport(p); f != nil {
		out = append(out, *f)
	}
	if f := routeSummary(p); f != nil {
		out = append(out, *f)
	}
	if f := parcelChecksum(p); f != nil {
		out = append(out, *f)
	}
	if f := deliveryProgressBucket(p, now); f != nil {
		out = append(out, *f)
	}
	return out
}

func strPtr(s string) *string { return &s }
func instantPtr(t time.Time) *Instant {
	v := Instant(t)
	return &v
}

func customsDocumentsRequired(p *ParcelResponse) *FeatureResponse {
	c := p.CustomsInformationRequirements
	if c == nil {
		return nil
	}
	if !c.DocumentsRequired || c.InformationProvided {
		return nil
	}
	pending := 0
	for _, e := range p.Events {
		if e.Type == "customs" && e.Cause != nil {
			pending++
		}
	}
	return &FeatureResponse{
		Type:        "CUSTOMS_DOCUMENTS_REQUIRED",
		Title:       strPtr("Customs information needed"),
		Description: strPtr(fmt.Sprintf("Pending customs events: %d", pending)),
	}
}

func heavyParcel(p *ParcelResponse) *FeatureResponse {
	if p.WeightInKg == nil {
		return nil
	}
	weight := float64(*p.WeightInKg)
	volumetric := 0.0
	if d := p.Dimensions; d != nil {
		volumetric = float64(d.LengthInCm) * float64(d.WidthInCm) * float64(d.HeightInCm) / volumetricDivisor
	}
	billable := weight
	if volumetric > billable {
		billable = volumetric
	}
	if billable <= heavyParcelKg {
		return nil
	}
	return &FeatureResponse{
		Type: "HEAVY_PARCEL",
		Description: strPtr(fmt.Sprintf(
			"Billable %.1f kg (actual %.1f, volumetric %.1f)",
			billable, weight, volumetric)),
	}
}

func rateDelivery(p *ParcelResponse) *FeatureResponse {
	if p.Status != "archived" || p.Direction != "receive" {
		return nil
	}
	var latest *time.Time
	for i := range p.Events {
		e := &p.Events[i]
		if e.Type == "delivered" || e.DisplayStatus == "delivered" {
			d := e.Date.Time()
			if latest == nil || d.After(*latest) {
				dd := d
				latest = &dd
			}
		}
	}
	feat := &FeatureResponse{
		Type: "RATE_DELIVERY",
		URL:  strPtr(fmt.Sprintf("https://posten.no/sporing/%s/rate", p.ParcelNumber)),
	}
	if latest != nil {
		feat.Date = instantPtr(*latest)
	}
	return feat
}

func pickupDeadlineSoon(p *ParcelResponse, now time.Time) *FeatureResponse {
	if p.Delivery == nil || p.Delivery.DeadlineDate == nil {
		return nil
	}
	deadline := p.Delivery.DeadlineDate.Time()
	remaining := deadline.Sub(now)
	if remaining < 0 || remaining >= pickupDeadlineWindowHours*time.Hour {
		return nil
	}
	hours := int64(remaining / time.Hour)
	dd := *p.Delivery.DeadlineDate
	return &FeatureResponse{
		Type:  "PICKUP_DEADLINE_SOON",
		Title: strPtr(fmt.Sprintf("Pick up within %d hours", hours)),
		Date:  &dd,
	}
}

func rewardsAvailable(p *ParcelResponse, now time.Time) *FeatureResponse {
	if p.Rewards == nil || len(p.Rewards.RewardsEarnings) == 0 {
		return nil
	}
	activeCoins := 0
	bestCoins := -1 << 31
	var bestType string
	haveBest := false
	for _, e := range p.Rewards.RewardsEarnings {
		from := e.ValidFrom.Time()
		to := e.ValidTo.Time()
		if now.Before(from) || !now.Before(to) {
			continue
		}
		activeCoins += e.Coins
		if e.Coins > bestCoins {
			bestCoins = e.Coins
			bestType = e.Type
			haveBest = true
		}
	}
	if activeCoins == 0 {
		return nil
	}
	feat := &FeatureResponse{
		Type:  "REWARDS_AVAILABLE",
		Title: strPtr(fmt.Sprintf("%d coins available", activeCoins)),
	}
	if haveBest {
		feat.Description = strPtr(fmt.Sprintf("Top reward: %s (%d coins)", bestType, bestCoins))
	}
	return feat
}

func latestEventCause(p *ParcelResponse) *FeatureResponse {
	if len(p.Events) == 0 {
		return nil
	}
	idx := 0
	for i := 1; i < len(p.Events); i++ {
		if p.Events[i].Date.Time().After(p.Events[idx].Date.Time()) {
			idx = i
		}
	}
	if p.Events[idx].Cause == nil {
		return nil
	}
	cause := *p.Events[idx].Cause
	date := p.Events[idx].Date
	return &FeatureResponse{
		Type:        "LATEST_EVENT_HAS_CAUSE",
		Description: &cause,
		Date:        &date,
	}
}

func greenTransport(p *ParcelResponse) *FeatureResponse {
	if p.Transport == nil || !p.Transport.Electric {
		return nil
	}
	return &FeatureResponse{
		Type:        "GREEN_TRANSPORT",
		Description: strPtr(fmt.Sprintf("Powered by %s", p.Transport.FuelType)),
	}
}

func routeSummary(p *ParcelResponse) *FeatureResponse {
	if len(p.Events) == 0 {
		return nil
	}
	idx := make([]int, len(p.Events))
	for i := range idx {
		idx[i] = i
	}
	sort.SliceStable(idx, func(a, b int) bool {
		return p.Events[idx[a]].Date.Time().Before(p.Events[idx[b]].Date.Time())
	})

	seen := make([]string, 0, len(idx))
	hash := hashSeed
	for _, i := range idx {
		e := &p.Events[i]
		if e.City == nil {
			continue
		}
		country := "??"
		if e.CountryCode != nil {
			country = *e.CountryCode
		}
		location := *e.City + "," + country
		dup := false
		for _, s := range seen {
			if s == location {
				dup = true
				break
			}
		}
		if dup {
			continue
		}
		// Iterate Unicode code points (rune) — matches Rust's
		// `for ch in location.chars()` so hashes match for non-ASCII cities.
		for _, r := range location {
			hash = (hash ^ uint64(r)) * hashPrime
		}
		seen = append(seen, location)
	}
	if len(seen) == 0 {
		return nil
	}
	route := strings.Join(seen, " -> ")
	return &FeatureResponse{
		Type:        "ROUTE_SUMMARY",
		URL:         strPtr(fmt.Sprintf("https://posten.no/sporing/%s#h%x", p.ParcelNumber, hash)),
		Title:       strPtr(fmt.Sprintf("%d stops", len(seen))),
		Description: &route,
	}
}

func parcelChecksum(p *ParcelResponse) *FeatureResponse {
	sum := 0
	weight := 1
	for _, r := range p.ParcelNumber {
		var digit int
		if r >= '0' && r <= '9' {
			digit = int(r - '0')
		} else {
			digit = int(r) % 10
		}
		sum += digit * weight
		if weight == 7 {
			weight = 1
		} else {
			weight += 2
		}
	}
	remainder := sum % checksumMod
	if remainder == 0 {
		return nil
	}
	return &FeatureResponse{
		Type:        "PARCEL_CHECKSUM_OK",
		Description: strPtr(fmt.Sprintf("checksum=%d", remainder)),
	}
}

func deliveryProgressBucket(p *ParcelResponse, now time.Time) *FeatureResponse {
	if len(p.Events) == 0 {
		return nil
	}
	first := p.Events[0].Date.Time()
	for _, e := range p.Events[1:] {
		if e.Date.Time().Before(first) {
			first = e.Date.Time()
		}
	}
	transitDays := int64(now.Sub(first) / (24 * time.Hour))
	if transitDays < 0 {
		transitDays = 0
	}
	statusWeight := 0
	switch p.Status {
	case "notified":
		statusWeight = 10
	case "underway":
		statusWeight = 30
	case "collectable":
		statusWeight = 60
	case "return_underway":
		statusWeight = 40
	case "return_collectable":
		statusWeight = 70
	case "archived":
		statusWeight = 100
	case "archived_by_user":
		statusWeight = 90
	}
	eventScore := len(p.Events) * 3
	if eventScore > 60 {
		eventScore = 60
	}
	timePenalty := int(transitDays)
	if timePenalty > 20 {
		timePenalty = 20
	}
	score := statusWeight + eventScore - timePenalty
	if score < 0 {
		score = 0
	}
	if score > 100 {
		score = 100
	}
	var bucket string
	switch {
	case score < 25:
		bucket = "early"
	case score < 60:
		bucket = "mid"
	case score < 90:
		bucket = "late"
	default:
		bucket = "complete"
	}
	return &FeatureResponse{
		Type:        "DELIVERY_PROGRESS_BUCKET",
		Title:       strPtr(bucket),
		Description: strPtr(fmt.Sprintf("score=%d transitDays=%d", score, transitDays)),
	}
}
