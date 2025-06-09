package auth

import (
    "time"
	"errors"
    "fmt"
    "os"
    "net/http"
    "crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/pem"
	"math/big"
    "github.com/gin-gonic/gin"
    "github.com/vivianlazaras/storyteller/model"
    "gorm.io/gorm"
	"golang.org/x/crypto/bcrypt"

    "github.com/golang-jwt/jwt/v5"
)

var jwtSigningKey *rsa.PrivateKey;
var JWTPubKey     rsa.PublicKey;

func RegisterAuthRoutes(r *gin.Engine) *gin.Engine {
    r.GET("/pubkey", GetJWTPubKey)
    return r
}

func InitAuth(path string) error {
    key, err := LoadJWTSigningKey(path)
    //fmt.Printf("loading private key: ", key)
    if err != nil {
        fmt.Printf("failed to load private key from file\n " + err.Error())
        return err
    }

    block, _ := pem.Decode(key)
	if block == nil || block.Type != "PRIVATE KEY" {
		return fmt.Errorf("operation failed: %s", "failed to decode private key " + block.Type)
	}

	parsedKey, err := x509.ParsePKCS8PrivateKey(block.Bytes)
    if err != nil {
        return fmt.Errorf("invalid PKCS#8 RSA private key: %w", err)
    }

    rsaKey, ok := parsedKey.(*rsa.PrivateKey)
    if !ok {
        return fmt.Errorf("parsed key is not an RSA private key")
    }

	pub := rsaKey.PublicKey
    jwtSigningKey = rsaKey
    JWTPubKey = pub
    return nil
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
    Sub     string  `json:"sub"`   // Subject (user ID)
    Email   string  `json:"email"` // Optional additional claim
    Iss     string  `json:"iss"`
    Aud     string  `json:"aud"`
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
        Aud:   "storyteller",
        Iss:    "http://localhost:8442",
        RegisteredClaims: jwt.RegisteredClaims{
            ExpiresAt: jwt.NewNumericDate(expiration),
            IssuedAt:  jwt.NewNumericDate(time.Now()),
        },
    }

    token := jwt.NewWithClaims(jwt.SigningMethodRS256, claims)
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

func GetJWTPubKey(c *gin.Context) {

	// Convert to JWK
	jwk := gin.H{
		"kty": "RSA",
		"n":   base64URL(JWTPubKey.N.Bytes()),
		"e":   base64URL(big.NewInt(int64(JWTPubKey.E)).Bytes()),
		"alg": "RS256",
		"use": "sig",
		"kid": "1", // key ID
	}

	c.JSON(http.StatusOK, gin.H{"keys": []any{jwk}})
}

// base64URL encodes to base64 URL encoding without padding
func base64URL(b []byte) string {
	return base64.RawURLEncoding.EncodeToString(b)
}