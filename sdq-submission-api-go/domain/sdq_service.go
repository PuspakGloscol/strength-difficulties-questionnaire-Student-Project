package domain

import (
	"time"

	"github.com/google/uuid"
)

type SdqClient struct { //creates a structure to show what data types the variables will be shown as
	ID        uuid.UUID `json:"id"`
	CodeName  string    `json:"codeName"`
	DoB       time.Time `json:"dateofbirth"`
	Gender    string    `json:"gender"`
	Council   string    `json:"council"`
	Ethnicity string    `json:"ethnicity"`
	EAL       string    `json:"eal"`
	DisStat   string    `json:"disability"`
	CareExp   string    `json:"careexp"`
	FundSrc   string    `json:"fundsrc"`
}

type SdqClientRepository interface { //creates the "interface" to be interacted with?
	GetClients() ([]SdqClient, error)
	CreateClient(
		codeName string,
		dateofbirth time.Time,
		gender string,
		council string,
		ethnicity string,
		eal string,
		disability string,
		careexp string,
		fundsrc string,
	) (SdqClient, error)
}

type SdqApp struct {
	ClientRepository SdqClientRepository
}
