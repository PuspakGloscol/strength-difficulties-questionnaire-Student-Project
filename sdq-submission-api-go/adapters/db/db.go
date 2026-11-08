package db

import (
	"context"
	"log"

	"github.com/google/uuid"

	"github.com/jackc/pgx/v5/pgxpool"
	domain "joesharpcs.co.uk/sdq/domain"

	"time"
)

type PgxClientRepository struct {
	DB *pgxpool.Pool
}

func (r *PgxClientRepository) GetClients() ([]domain.SdqClient, error) {
	rows, err := r.DB.Query(context.Background(),
		"SELECT client_id, code_name, date_of_birth, gender, council, ethnicity, eal, disability_status, care_experience, funding_source FROM client", //SQL query to select data from db
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var out []domain.SdqClient

	for rows.Next() {
		var c domain.SdqClient
		if err := rows.Scan(
			&c.ID, 
			&c.CodeName, 
			&c.DoB, 
			&c.Gender, 
			&c.Council,
			&c.Ethnicity,
			&c.EAL,
			&c.DisStat,
			&c.CareExp,
			&c.FundSrc,
			); err != nil { //Scans rows/columns in db to extract data *
			return nil, err
		}
		out = append(out, c)
	}

	return out, nil
}

func (r *PgxClientRepository) CreateClient(
	codeName string, 
	dateofbirth time.Time, 
	gender string, 
	council string,
	ethnicity string,
	eal string,
	disability string,
	careexp string,
	fundsrc string,
	) (domain.SdqClient, error) {//creates a client to be displayed using the data from the db
	var id uuid.UUID

	err := r.DB.QueryRow(context.Background(),
		"INSERT INTO client (code_name, date_of_birth, gender, council, ethnicity, eal, disability_status, care_experience, funding_source) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING client_id", //change this
		codeName,
		dateofbirth,
		gender,
		council,
		ethnicity,
		eal,
		disability,
		careexp,
		fundsrc,
	).Scan(&id)

	if err != nil {
		return domain.SdqClient{}, err
	}

	return domain.SdqClient{ // returns the data from the BE to be displayed?
		ID:       id,
		CodeName: codeName,
		DoB: dateofbirth,
		Gender: gender,
		Council: council,
		Ethnicity: ethnicity,
		EAL: eal,
		DisStat: disability,
		CareExp: careexp,
		FundSrc: fundsrc,

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
