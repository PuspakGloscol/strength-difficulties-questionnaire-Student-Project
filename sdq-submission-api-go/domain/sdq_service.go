package domain

import (
	"github.com/google/uuid"
)

type SdqClient struct {
	ID       uuid.UUID `json:"id"`
	CodeName string    `json:"codeName"`
}

type SdqClientRepository interface {
	GetClients() ([]SdqClient, error)
	CreateClient(codeName string) (SdqClient, error)
}

type SdqApp struct {
	ClientRepository SdqClientRepository
}
