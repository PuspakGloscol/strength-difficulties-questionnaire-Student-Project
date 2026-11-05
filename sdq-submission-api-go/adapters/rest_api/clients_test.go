package rest

import (
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/google/uuid"
	domain "joesharpcs.co.uk/sdq/domain"

	"time"
)

type fakeRepo struct {
	clients []domain.SdqClient
	err     error
}

func (f fakeRepo) GetClients() ([]domain.SdqClient, error) { return f.clients, f.err }
func (f fakeRepo) CreateClient(string) (domain.SdqClient, error) {
	return domain.SdqClient{}, nil
}

func get(repo domain.SdqClientRepository) *httptest.ResponseRecorder {
	rec := httptest.NewRecorder()
	CreateRestApi(repo).ServeHTTP(rec, httptest.NewRequest("GET", "/api/clients", nil))
	return rec
}

func TestGetClients(t *testing.T) {
	id := uuid.New()
	rec := get(fakeRepo{clients: []domain.SdqClient{{
		ID: id, 
		CodeName: "abc", 
		DoB: "abc", 
		Gender: "abc", 
		Council: "abc",
		Ethnicity: "abc",
		EAL: "abc",
		DisStat: "abc",
		CareExp: "abc",
		FundSrc: "abc"
		}}}) //not entirely sure what this does? Gets the client data??
	if rec.Code != http.StatusOK {
		t.Fatalf("status %d", rec.Code)
	}
	want := `[{"id":"` + id.String() + `","codeName":"abc", "dateofbirth":"abc", "gender":"abc", "council":"abc", "ethnicity": "abc", "eal": "abc", "disability": "abc", "careexp": "abc", "fundsrc": "abc"}]` //show what data the test wants to not fail
	if strings.TrimSpace(rec.Body.String()) != want {
		t.Fatalf("got %s", rec.Body.String())
	}
}

func TestGetClientsEmptyIsArray(t *testing.T) {
	if body := strings.TrimSpace(get(fakeRepo{}).Body.String()); body != "[]" {
		t.Fatalf("got %s", body)
	}
}

func TestGetClientsError(t *testing.T) {
	if code := get(fakeRepo{err: errors.New("boom")}).Code; code != http.StatusInternalServerError {
		t.Fatalf("status %d", code)
	}
}
