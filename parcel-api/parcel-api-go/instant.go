package main

import (
	"errors"
	"fmt"
	"strconv"
	"strings"
	"time"
)

// Float is float64 with a JSON encoding that always preserves the decimal
// point, matching serde_json's behaviour (`4.0` stays `4.0`, not `4`).
type Float float64

func (f Float) MarshalJSON() ([]byte, error) {
	s := strconv.FormatFloat(float64(f), 'g', -1, 64)
	if !strings.ContainsAny(s, ".eE") {
		s += ".0"
	}
	return []byte(s), nil
}

func (f *Float) UnmarshalJSON(data []byte) error {
	v, err := strconv.ParseFloat(string(data), 64)
	if err != nil {
		return err
	}
	*f = Float(v)
	return nil
}

// Instant mirrors the Rust `instant_format` serializer: an ISO 8601 UTC
// timestamp where the fractional digits are dropped when zero and otherwise
// padded to 3, 6, or 9 digits depending on precision. This matches Java's
// `Instant.toString()`, which is what the JVM ports emit.
type Instant time.Time

func (i Instant) MarshalJSON() ([]byte, error) {
	t := time.Time(i).UTC()
	ns := t.Nanosecond()
	base := t.Format("2006-01-02T15:04:05")
	var frac string
	switch {
	case ns == 0:
		frac = ""
	case ns%1_000_000 == 0:
		frac = fmt.Sprintf(".%03d", ns/1_000_000)
	case ns%1_000 == 0:
		frac = fmt.Sprintf(".%06d", ns/1_000)
	default:
		frac = fmt.Sprintf(".%09d", ns)
	}
	return []byte(`"` + base + frac + `Z"`), nil
}

func (i *Instant) UnmarshalJSON(data []byte) error {
	s := strings.Trim(string(data), `"`)
	if s == "" || s == "null" {
		return errors.New("empty instant")
	}
	// Accept any RFC3339 variant; the input files use whole seconds with `Z`.
	t, err := time.Parse(time.RFC3339Nano, s)
	if err != nil {
		return err
	}
	*i = Instant(t.UTC())
	return nil
}

func (i Instant) Time() time.Time { return time.Time(i) }
