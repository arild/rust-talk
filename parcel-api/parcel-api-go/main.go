package main

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"syscall"
	"time"
)

const (
	defaultPort    = "8080"
	defaultDataDir = "/parcel-data"
)

func main() {
	dataDir := os.Getenv("PARCEL_DATA_DIR")
	if dataDir == "" {
		dataDir = defaultDataDir
	}
	log.Printf("loading parcel data from %s", dataDir)
	svc, err := loadService(dataDir)
	if err != nil {
		log.Fatalf("service load: %v", err)
	}
	log.Printf("loaded %d parcels", svc.count())

	mux := http.NewServeMux()
	// Order doesn't matter for ServeMux pattern matching — Go 1.22 picks the
	// most-specific route on its own.
	mux.HandleFunc("GET /parcel-api/check/status", handleCheckStatus)
	mux.HandleFunc("GET /parcel-api/check", handleCheck)
	mux.HandleFunc("POST /parcel-api/v1/parcel", listParcelsHandler(svc))

	srv := &http.Server{
		Addr:    ":" + defaultPort,
		Handler: mux,
	}
	go func() {
		log.Printf("parcel-api-go listening on :%s", defaultPort)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("server: %v", err)
		}
	}()

	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	<-sigCh
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	_ = srv.Shutdown(ctx)
}

func handleCheckStatus(w http.ResponseWriter, _ *http.Request) {
	w.Header().Set("Content-Type", "text/plain; charset=UTF-8")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("👋 parcel-api is on air"))
}

func handleCheck(w http.ResponseWriter, _ *http.Request) {
	mem := memorySummary()
	body := "parcel-api\n\nMemory:\n" + mem + "\n\nVersion:\ndev\n"
	w.Header().Set("Content-Type", "text/plain; charset=UTF-8")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(body))
}

// Reads /proc/self/statm — same approach as the Rust /check endpoint.
func memorySummary() string {
	f, err := os.Open("/proc/self/statm")
	if err != nil {
		return "memory stats unavailable on this platform"
	}
	defer f.Close()
	sc := bufio.NewScanner(f)
	if !sc.Scan() {
		return "memory stats unavailable on this platform"
	}
	parts := strings.Fields(sc.Text())
	if len(parts) < 2 {
		return "memory stats unavailable on this platform"
	}
	totalPages, _ := strconv.ParseUint(parts[0], 10, 64)
	rssPages, _ := strconv.ParseUint(parts[1], 10, 64)
	const pageBytes = 4096
	return fmt.Sprintf("total: %dmb, resident: %dmb",
		totalPages*pageBytes/1_048_576,
		rssPages*pageBytes/1_048_576)
}

func listParcelsHandler(svc *service) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		// All ports ignore the request body; drain it so the
		// keep-alive connection can be reused.
		if r.Body != nil {
			_, _ = io.Copy(io.Discard, r.Body)
			r.Body.Close()
		}

		data, err := svc.listParcels()
		if err != nil {
			sendError(w, http.StatusInternalServerError, err.Error())
			return
		}
		writeJSON(w, http.StatusOK, data)
	}
}

type errorResponse struct {
	Message string `json:"message"`
}

func sendError(w http.ResponseWriter, status int, message string) {
	writeJSON(w, status, errorResponse{Message: message})
}

func writeJSON(w http.ResponseWriter, status int, body any) {
	// Custom Encoder with HTML-escape off so &/<,> survive as-is —
	// matches serde_json's default. The Encoder also appends a trailing
	// newline that we strip for byte parity with Rust.
	var buf bytes.Buffer
	enc := json.NewEncoder(&buf)
	enc.SetEscapeHTML(false)
	if err := enc.Encode(body); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	out := buf.Bytes()
	if n := len(out); n > 0 && out[n-1] == '\n' {
		out = out[:n-1]
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_, _ = w.Write(out)
}
