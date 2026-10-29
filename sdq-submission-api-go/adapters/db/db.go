package db

import (
	"context"
	"log"

	"github.com/google/uuid"

	"github.com/jackc/pgx/v5/pgxpool"
	domain "joesharpcs.co.uk/sdq/domain"
)

type PgxClientRepository struct {
	DB *pgxpool.Pool
}

func (r *PgxClientRepository) GetClients() ([]domain.SdqClient, error) {
	rows, err := r.DB.Query(context.Background(),
		"SELECT client_id, code_name FROM client",
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var out []domain.SdqClient

	for rows.Next() {
		var c domain.SdqClient
		if err := rows.Scan(&c.ID, &c.CodeName); err != nil {
			return nil, err
		}
		out = append(out, c)
	}

	return out, nil
}

func (r *PgxClientRepository) CreateClient(codeName string) (domain.SdqClient, error) {
	var id uuid.UUID

	err := r.DB.QueryRow(context.Background(),
		"INSERT INTO client (code_name) VALUES ($1) RETURNING client_id",
		codeName,
	).Scan(&id)

	if err != nil {
		return domain.SdqClient{}, err
	}

	return domain.SdqClient{
		ID:       id,
		CodeName: codeName,
	}, nil
}

func CreateDb(db_url string) domain.SdqClientRepository {
	pool, err := pgxpool.New(context.Background(), db_url)
	if err != nil {
		log.Fatalf("unable to connect to database: %v", err)
	}

	if err := pool.Ping(context.Background()); err != nil {
		log.Fatalf("unable to connect to database: %v", err)
	}

	return &PgxClientRepository{DB: pool}
}
