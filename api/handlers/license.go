package handlers

import (
	"github.com/vivianlazaras/storyteller/model"
	"github.com/google/uuid"
)



func LoadLicenses() {

}

func InsertLicense(license *model.License) {
	if license.ID == "" {
		license.ID = uuid.New().String()
	}


}