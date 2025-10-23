package handlers

import (
	"github.com/vivianlazaras/storyteller/model"
	// "github.com/vivianlazaras/storyteller/handlers"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
)

func defaultMetadata(creatorID uuid.UUID) model.Metadatum {

	return model.Metadatum {
		ID: uuid.New(),
		Creator: &creatorID,
		License: nil,
		Public: false,
	}
}

func createMetadata(metadata *model.Metadatum) error {
	err := db.DB.Create(&metadata).Error
	return err
}

func createDefaultMetadata(creatorID uuid.UUID) (model.Metadatum, error) {
	var metadata = defaultMetadata(creatorID)
	/*license, err := createDefaultLicense()
	if err != nil {
		return model.Metadatum{}, err
	}*/

	metadata.License = nil
	merr := createMetadata(&metadata)
	return metadata, merr
}