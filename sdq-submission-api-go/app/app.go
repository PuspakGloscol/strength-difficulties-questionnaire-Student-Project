package main

import (
	"fmt"
	"net/http"
	"os"

	"github.com/go-chi/chi/v5"
	db "joesharpcs.co.uk/sdq/db"
	domain "joesharpcs.co.uk/sdq/domain"
	rest "joesharpcs.co.uk/sdq/rest_api"
)

func main() {
	submission_dir := os.Getenv("SUBMISSION_UI_RESOURCES_DIR")
	db_url := os.Getenv("DATABASE_URL")

	app := domain.SdqApp{
		ClientRepository: db.CreateDb(db_url),
	}

	fmt.Println("Hello Joe SDQ App", app)
	fmt.Println("Hosting Static resources from", submission_dir)
	fmt.Println("Connecting to Database at", db_url)
	db.CreateDb(db_url)
	r := rest.CreateRestApi()
	fileServer(r, "", http.Dir(submission_dir))
	http.ListenAndServe(":3000", r)
}

func fileServer(r chi.Router, path string, root http.FileSystem) {
	fs := http.StripPrefix(path, http.FileServer(root))

	r.Get(path+"/*", func(w http.ResponseWriter, r *http.Request) {
		fs.ServeHTTP(w, r)
	})
}
