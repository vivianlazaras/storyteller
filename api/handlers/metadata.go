package handlers

import (
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
)

func CreateMetadata(creatorID, license uuid.UUID, public bool) (model.Metadatum, error) {
	var metadata = model.Metadatum {
		ID: uuid.New().String(),
		Creator: creatorID.String(),
		License: license.String(),
		Shared: "",
		Public: public,
	}

	err := db.DB.Create(&metadata).Error
	return metadata, err
}