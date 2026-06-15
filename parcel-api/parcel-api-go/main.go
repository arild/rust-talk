package main

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"os"
	"os/signal"
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
	mux.HandleFunc("GET /parcel-api/parcel", listParcelsHandler(svc))

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
