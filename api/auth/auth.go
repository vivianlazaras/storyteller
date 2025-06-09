package auth

import (
    "time"
	"errors"
    "fmt"
    "os"
    "github.com/vivianlazaras/storyteller/model"
    "gorm.io/gorm"
	"golang.org/x/crypto/bcrypt"
    "github.com/golang-jwt/jwt/v5"
)

var jwtSigningKey []byte;
func InitAuth(path string) error {
    key, err := LoadJWTSigningKey(path)
    jwtSigningKey = key
    return err
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

type OIDCClaims struct {
    Sub   string `json:"sub"`   // Subject (user ID)
    Email string `json:"email"` // Optional additional claim
    jwt.RegisteredClaims        // includes exp, iat, etc.
}

func AuthenticateAndIssueToken(db *gorm.DB, email, password string) (string, error) {
    user, err := GetUserByEmail(db, email)
    if err != nil {
        return "", err
    }

    // Verify password
    if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(password)); err != nil {
        return "", errors.New("invalid password")
    }

    // Create OIDC-style claims
    expiration := time.Now().Add(time.Hour)
    claims := OIDCClaims{
        Sub:   user.ID,
        Email: user.Email,
        RegisteredClaims: jwt.RegisteredClaims{
            ExpiresAt: jwt.NewNumericDate(expiration),
            IssuedAt:  jwt.NewNumericDate(time.Now()),
        },
    }

    token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
    signedToken, err := token.SignedString(jwtSigningKey)
    if err != nil {
        return "", err
    }

    return signedToken, nil
}

// LoadJWTSigningKey reads the signing key from the given file path.
func LoadJWTSigningKey(filePath string) ([]byte, error) {
    key, err := os.ReadFile(filePath)
    if err != nil {
        return nil, fmt.Errorf("failed to load JWT signing key: %w", err)
    }

    return key, nil
}