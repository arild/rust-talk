package main

import (
	"encoding/json"
	"fmt"
	"math/rand/v2"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// Stores parcel JSON as raw bytes in memory; re-deserialises on every
// request. Mirrors the Rust StubParcelService — putting the JSON parse cost
// on the request path so the bench measures the work a real adapter does
// when decoding data from a downstream.
type service struct {
	parcels [][]byte // raw JSON, sorted by parcel number
	numbers []string // parcel numbers in the same order
}

func loadService(dir string) (*service, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("read parcel data dir %q: %w", dir, err)
	}
	sort.Slice(entries, func(i, j int) bool { return entries[i].Name() < entries[j].Name() })

	svc := &service{}
	for _, e := range entries {
		name := e.Name()
		if !strings.HasSuffix(name, ".json") {
			continue
		}
		bytes, err := os.ReadFile(filepath.Join(dir, name))
		if err != nil {
			return nil, fmt.Errorf("read %s: %w", name, err)
		}
		stem := strings.TrimSuffix(name, ".json")
		svc.numbers = append(svc.numbers, stem)
		svc.parcels = append(svc.parcels, bytes)
	}
	return svc, nil
}

func (s *service) count() int { return len(s.parcels) }

func (s *service) listParcels() ([]ParcelResponse, error) {
	now := time.Now().UTC()
	order := make([]int, len(s.parcels))
	for i := range order {
		order[i] = i
	}
	rand.Shuffle(len(order), func(i, j int) { order[i], order[j] = order[j], order[i] })

	parcels := make([]ParcelResponse, 0, len(order))
	for _, i := range order {
		var p ParcelResponse
		if err := json.Unmarshal(s.parcels[i], &p); err != nil {
			return nil, fmt.Errorf("parse parcel %s: %w", s.numbers[i], err)
		}
		p.Features = computeFeatures(&p, now)
		parcels = append(parcels, p)
	}
	return parcels, nil
}
