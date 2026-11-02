package rest

import (
	"encoding/json"
	"log"
	"net/http"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	domain "joesharpcs.co.uk/sdq/domain"
)

type greetRequest struct {
	Name string `json:"name"`
}

type greetResponse struct {
	Name     string `json:"name"`
	Greeting string `json:"greeting"`
}

func CreateRestApi(clients domain.SdqClientRepository) *chi.Mux {
	r := chi.NewRouter()
	r.Use(middleware.Logger)

	// Mount API routes under /api
	r.Route("/api", func(api chi.Router) {
		api.Get("/health", func(w http.ResponseWriter, r *http.Request) {
			log.Println("Health check endpoint called")
			w.Write([]byte("ok"))
		})

		api.Post("/greet", func(w http.ResponseWriter, r *http.Request) {
			var req greetRequest
			if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
				log.Printf("failed to decode request body: %v", err)
				http.Error(w, "invalid JSON body", http.StatusBadRequest)
				return
			}
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(greetResponse{Name: req.Name, Greeting: "Hello " + req.Name})
		})

		api.Route("/clients", func(c chi.Router) {
			c.Get("/", func(w http.ResponseWriter, r *http.Request) {
				list, err := clients.GetClients()
				if err != nil {
					log.Printf("failed to get clients: %v", err)
					http.Error(w, "failed to retrieve clients", http.StatusInternalServerError)
					return
				}
				if list == nil {
					list = []domain.SdqClient{}
				}
				w.Header().Set("Content-Type", "application/json")
				json.NewEncoder(w).Encode(list)
			})
		})
	})

	return r
}
