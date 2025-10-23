package handlers

import (
	"errors"
    "log"
    "fmt"
    "golang.org/x/crypto/bcrypt"
    "net/http"
    "github.com/vivianlazaras/storyteller/model"
    "github.com/vivianlazaras/storyteller/auth"
    "github.com/vivianlazaras/storyteller/db"
    "gorm.io/gorm"
    "github.com/gin-gonic/gin"
    "github.com/google/uuid"
)

type UserRender struct {
    ID              string `json:"id"`
    FamilyName      string `json:"family_name"`
    GivenName       string  `json:"given_name"`
    Email           string  `json:"email"`
    Subject         *uuid.UUID  `json:"subject"`
    Gender          string `json:"gender"`
    Sex             string  `json:"sex"`
    DefaultGroup    uuid.UUID `json:"default_group"`
    ProfileImage    *string  `json:"profile_image"` 
}

type CreateAccount struct {
    Email           string  `json:"email"`
    Password        string  `json:password"`
    VerifyPassword  string  `json:"verify_password"`
    FirstName       string  `json:"fname"`
    LastName        string  `json:"lname"`
    Gender          string  `json:"gender"`
}

func RegisterProfileRoutes(r *gin.Engine) *gin.Engine {
    r.POST("/login", Login)
    r.POST("/accounts/local/create", CreateLocalAccount)
    r.POST("update_password", ChangePassword)
    r.GET("/profiles/info", auth.JWTMiddleware(), ProfileInfo)
    return r
}

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

func Login(c *gin.Context) {
    var login LoginInfo
    if jsonerr := c.ShouldBindJSON(&login); jsonerr != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse input as json"})
        return
    }

    token, autherr := auth.AuthenticateAndIssueToken(db.DB, login.Email, login.Password)
    if autherr != nil {
        c.JSON(http.StatusUnauthorized, autherr.Error())
        log.Println("ERROR: " + autherr.Error())
        return
    }

    c.JSON(http.StatusOK, token)
}

func CreateNewAccount(tx *gorm.DB, builder CreateAccount, subject uuid.UUID) (model.User, error) {
    //var description = builder.Email + "'s default group";
    if builder.Password != builder.VerifyPassword {
        return model.User{}, fmt.Errorf("passwords don't match");
    }

    hashedPassword, err := bcrypt.GenerateFromPassword([]byte(builder.Password), bcrypt.DefaultCost)
    if err != nil {
        return model.User{}, fmt.Errorf("failed to hash password: %w", err)
    }

    var user = model.User {
        Email:      builder.Email,
        Fname:      builder.FirstName,
        Lname:      builder.LastName,
        Subject:    &subject,
        Gender:     builder.Gender,
        PasswordHash: strPtr(string(hashedPassword)),
    }

    group, err := CreateDefaultGroup(tx, user.ID, user.Email + "_default")
    if err != nil {
        return model.User{}, err
    }

    user.DefaultGroup = group.ID
    if err := tx.Create(&user).Error; err != nil {
		return model.User{}, err
	}

    return user, nil
}

/// this route is different from CreateAccount as it sets a subject
func CreateLocalAccount(c *gin.Context) {
    var accountData CreateAccount
    subject := uuid.New()

    if err := c.ShouldBindJSON(&accountData); err != nil {
        c.JSON(http.StatusBadRequest, gin.H{"error": "malformed API call"})
        return
    }

    tx := db.DB.Begin()
	if tx.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create transaction"})
		return
	}

    user, err := CreateNewAccount(tx, accountData, subject);
    if err != nil {
        tx.Rollback()
        c.JSON(http.StatusBadRequest, gin.H{"error": err});
        return
    }



    c.JSON(http.StatusOK, user)
}

// Request payload
type ChangePasswordRequest struct {
	OldPassword     string `json:"old_password" binding:"required"`
	NewPassword     string `json:"new_password" binding:"required"`
	ConfirmPassword string `json:"confirm_password" binding:"required"`
}

func ChangePassword(c *gin.Context) {
	var req ChangePasswordRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request: " + err.Error()})
		return
	}

	if req.NewPassword != req.ConfirmPassword {
		c.JSON(http.StatusBadRequest, gin.H{"error": "New passwords do not match"})
		return
	}

	
	// Get user from claims
	user, err := auth.GetUserFromClaims(db.DB, c)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Unauthorized: " + err.Error()})
		return
	}

	// Sanity check
	if user.PasswordHash == nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "User has no password set"})
		return
	}

	// Check old password
	if err := bcrypt.CompareHashAndPassword([]byte(*user.PasswordHash), []byte(req.OldPassword)); err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "Old password is incorrect"})
		return
	}

	// Hash new password
	hashed, err := bcrypt.GenerateFromPassword([]byte(req.NewPassword), bcrypt.DefaultCost)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to hash new password: " + err.Error()})
		return
	}

	// Update password in DB
	if err := db.DB.Model(&user).Update("password_hash", string(hashed)).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to update password: " + err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "Password changed successfully"})
}

func ProfileInfo(c *gin.Context) {
    user, err := auth.GetUserFromClaims(db.DB, c)
    if err != nil {
        c.JSON(http.StatusUnauthorized, err)
        return
    }

    var render = UserRender {
        GivenName: user.Fname,
        FamilyName: user.Lname,
        Email: user.Email,
        Subject: user.Subject,
        Sex:    user.Sex,
        Gender: user.Gender,
        DefaultGroup: user.DefaultGroup,
        ProfileImage:   user.ProfileImage,
    }
    PrintObjDebug(render)
    c.JSON(http.StatusOK, render)
}