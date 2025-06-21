package auth

import (
    "time"
	"errors"
    "fmt"
    "os"
    "strings"
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

// GetUserIDFromContext extracts the user ID from OIDC claims in Gin context
func GetUserIDFromContext(c *gin.Context) (string, error) {
	claimsVal, exists := c.Get("claims")
	if !exists {
		return "", errors.New("no claims found in context")
	}

	claims, ok := claimsVal.(*OIDCClaims)
	if !ok {
		return "", errors.New("invalid claims type in context")
	}

	return claims.Sub, nil
}

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
        Sub:   user.Subject,
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

// ValidateToken verifies the JWT's signature and standard claims
func ValidateToken(tokenString string) (*OIDCClaims, error) {
	// Parse token and validate signature
	token, err := jwt.ParseWithClaims(tokenString, &OIDCClaims{}, func(token *jwt.Token) (any, error) {
		// Ensure the token uses the expected signing method
		if _, ok := token.Method.(*jwt.SigningMethodRSA); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return &JWTPubKey, nil
	})
	if err != nil {
		return nil, fmt.Errorf("token parsing failed: %w", err)
	}

	// Validate claims
	claims, ok := token.Claims.(*OIDCClaims)
	if !ok || !token.Valid {
		return nil, errors.New("invalid token or claims")
	}

	// Additional OIDC-style checks (issuer, audience, expiration)
	if claims.Iss != "http://localhost:8442" {
		return nil, fmt.Errorf("invalid issuer: %s", claims.Iss)
	}
	if claims.Aud != "storyteller" {
		return nil, fmt.Errorf("invalid audience: %s", claims.Aud)
	}

	return claims, nil
}

// JWTMiddleware validates the JWT and injects claims into the Gin context
func JWTMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Get the Authorization header
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" || !strings.HasPrefix(authHeader, "Bearer ") {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "missing or malformed Authorization header"})
			return
		}

		// Extract token string
		tokenString := strings.TrimPrefix(authHeader, "Bearer ")

		// Validate token
		claims, err := ValidateToken(tokenString)
		if err != nil {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": err.Error()})
			return
		}

		
		// Store claims in the context for handlers to access
		c.Set("claims", claims)
		/*
		if c.Request.Method == http.MethodPut || c.Request.Method == http.MethodDelete {
			idParam := c.Param("id")
			if idParam == "" {
				c.AbortWithStatusJSON(http.StatusBadRequest, gin.H{"error": "missing resource ID in path"})
				return
			}

			user, err := GetUserFromClaims(db.DB, c)
			if err != nil {
				c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "unauthorized: " + err.Error()})
				return
			}

			requiredPerm := ""
			if c.Request.Method == http.MethodPut {
				requiredPerm = "update"
			} else if c.Request.Method == http.MethodDelete {
				requiredPerm = "delete"
			}

			var count int64
			err = db.DB.
				Table("entities").
				Select("count(*)").
				Joins("JOIN grouprel ON entities.group_id = grouprel.group_id").
				Joins("JOIN groups ON entities.group_id = groups.id").
				Where("entities.id = ? AND grouprel.user_id = ?", idParam, user.ID).
				Where("? = ANY(groups.permissions)", requiredPerm). // Postgres array contains
				Count(&count).Error

			if err != nil {
				c.AbortWithStatusJSON(http.StatusInternalServerError, gin.H{"error": "database error"})
				return
			}

			if count == 0 {
				c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "permission denied"})
				return
			}
		}*/

		c.Next()
	}
}

// GetUserByEmailFromClaims retrieves the user from the DB using the email in OIDC claims
func GetUserFromClaims(db *gorm.DB, c *gin.Context) (*model.User, error) {
	claimsVal, exists := c.Get("claims")
	if !exists {
		return nil, errors.New("no claims found in context")
	}

	claims, ok := claimsVal.(*OIDCClaims)
	if !ok {
		return nil, errors.New("invalid claims format in context")
	}

	var user model.User
	result := db.Where("email = ? and subject = ?", claims.Email, claims.Sub).First(&user)
	if errors.Is(result.Error, gorm.ErrRecordNotFound) {
		return nil, errors.New("user not found")
	} else if result.Error != nil {
		return nil, result.Error
	}
	return &user, nil
}