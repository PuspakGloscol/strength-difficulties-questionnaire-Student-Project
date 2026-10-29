package domain

import (
	"github.com/google/uuid"
)

type SdqClient struct {
	ID       uuid.UUID
	CodeName string
}

type SdqClientRepository interface {
	GetClients() ([]SdqClient, error)
	CreateClient(codeName string) (SdqClient, error)
}

type SdqApp struct {
	ClientRepository SdqClientRepository
}
