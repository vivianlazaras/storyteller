package handlers

import (
	"errors"
    "github.com/vivianlazaras/storyteller/model"
    "gorm.io/gorm"
)

type LoginInfo struct {
	Email		string `json:"email"`
	// this should be an already hashed password at this point
	Password	string	`json:"password"`
}

func GetUserByEmail(db *gorm.DB, email string) (*model.User, error) {
    var user model.User
    result := db.Where("email = ?", email).First(&user)
    if errors.Is(result.Error, gorm.ErrRecordNotFound) {
        return nil, errors.New("user not found")
    } else if result.Error != nil {
        return nil, result.Error
    }
    return &user, nil
}