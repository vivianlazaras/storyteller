package handlers

import (
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
)

/*func defaultLicense() model.License {
	return model.License {
		ID: uuid.New().String(),
		Name: "MIT",
		Description: "MIT Open Source License",
		Public: true,
		Content: "",
	}
}*/

func createLicense(license *model.License) error {
	err := db.DB.Create(license).Error
	return err
}

/*func createDefaultLicense() (model.License, error) {
	var license = defaultLicense()
	err := createLicense(&license)
	return license, err
}*/
func LoadLicenses() {

}

func InsertLicense(license *model.License) {
	


}