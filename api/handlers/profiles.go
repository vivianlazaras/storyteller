package handlers

import (
	"errors"
    "net/http"
    "github.com/vivianlazaras/storyteller/model"
    "github.com/vivianlazaras/storyteller/auth"
    "github.com/vivianlazaras/storyteller/db"
    "gorm.io/gorm"
    "github.com/gin-gonic/gin"
)

func RegisterProfileRoutes(r *gin.Engine) *gin.Engine {
    r.POST("/login", Login)
    return r
}

type LoginInfo struct {
	Email		string `json:"email"`
	// this should be an already hashed password at this point
	Password	string	`json:"password"`
}

func GetUserByEmail(email string) (*model.User, error) {
    var user model.User
    result := db.DB.Where("email = ?", email).First(&user)
    if errors.Is(result.Error, gorm.ErrRecordNotFound) {
        return nil, errors.New("user not found")
    } else if result.Error != nil {
        return nil, result.Error
    }
    return &user, nil
}

func Login(c *gin.Context) {
    var login LoginInfo
    if jsonerr := c.ShouldBindJSON(&login); jsonerr != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse input as json"})
        return
    }

    token, autherr := auth.AuthenticateAndIssueToken(db.DB, login.Email, login.Password)
    if autherr != nil {
        c.JSON(http.StatusUnauthorized, gin.H{"error": "failed to verify user authenticity"})
        return
    }

    c.JSON(http.StatusOK, token)
}