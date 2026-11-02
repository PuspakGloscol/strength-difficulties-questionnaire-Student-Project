package rest

import (
	"encoding/json"
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
)

type greetRequest struct {
	Name string `json:"name"`
}

type greetResponse struct {
	Name     string `json:"name"`
	Greeting string `json:"greeting"`
}

func CreateRestApi() *chi.Mux {
	r := chi.NewRouter()
	r.Use(middleware.Logger)

	// Mount API routes under /api
	r.Route("/api", func(api chi.Router) {
		api.Get("/health", func(w http.ResponseWriter, r *http.Request) {
			w.Write([]byte("ok"))
		})

		api.Post("/greet", func(w http.ResponseWriter, r *http.Request) {
			var req greetRequest
			if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
				http.Error(w, "invalid JSON body", http.StatusBadRequest)
				return
			}
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(greetResponse{Name: req.Name, Greeting: "Hello " + req.Name})
		})
	})

	return r
}
