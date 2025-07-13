package auth

import (
    "time"
	"errors"
	"encoding/json"
	"encoding/base64"
    "fmt"
    "os"
	"sync"
    "strings"
    "net/http"
    "crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"math/big"
    "github.com/gin-gonic/gin"
    "github.com/vivianlazaras/storyteller/model"
    "github.com/vivianlazaras/storyteller/config"
    "gorm.io/gorm"
	"golang.org/x/crypto/bcrypt"
	"github.com/google/uuid"
    "github.com/golang-jwt/jwt/v5"
)

var jwtSigningKey *rsa.PrivateKey;
var JWTPubKey     rsa.PublicKey;
var mu sync.Mutex

// GetUserIDFromContext extracts the user ID from OIDC claims in Gin context
func GetUserIDFromContext(c *gin.Context) (*uuid.UUID, error) {
	claimsVal, exists := c.Get("claims")
	if !exists {
		return nil, errors.New("no claims found in context")
	}

	claims, ok := claimsVal.(*OIDCClaims)
	if !ok {
		return nil, errors.New("invalid claims type in context")
	}

	return claims.Sub, nil
}

func RegisterAuthRoutes(r *gin.Engine) *gin.Engine {
    r.GET("/pubkey", GetJWTPubKey)
    return r
}

// JWK represents a simplified JSON Web Key (you can extend this if needed).
type JWK struct {
	Kid string `json:"kid"`
	Kty string `json:"kty"`
	Alg string `json:"alg"`
	Use string `json:"use"`
	N   string `json:"n,omitempty"`
	E   string `json:"e,omitempty"`
	// Add other fields if your use case requires
}

// JWKS is a JSON Web Key Set.
type JWKS struct {
	Keys []JWK `json:"keys"`
}

// DiscoveryDoc represents the OIDC discovery document.
type DiscoveryDoc struct {
	JWKSURI string `json:"jwks_uri"`
}

var oidcPubKeys map[string]map[string][]JWK
var localIssuer string = "http://localhost:8442"
var localKid string = "primary"
var localAlg string = "RS256"

func BuildRSAPublicKey(nStr, eStr string) (*rsa.PublicKey, error) {
	nBytes, err := base64.RawURLEncoding.DecodeString(nStr)
	if err != nil {
		return nil, fmt.Errorf("failed to decode N: %v", err)
	}
	eBytes, err := base64.RawURLEncoding.DecodeString(eStr)
	if err != nil {
		return nil, fmt.Errorf("failed to decode E: %v", err)
	}
	n := new(big.Int).SetBytes(nBytes)

	// e is usually small: 65537 (0x10001)
	e := new(big.Int).SetBytes(eBytes).Int64()
	return &rsa.PublicKey{N: n, E: int(e)}, nil
}

// GetOIDCPubKeys discovers and stores the JWKS keys for each issuer.
func GetOIDCPubKeys(configs []config.OIDCConfig) {
	client := &http.Client{
		Timeout: 10 * time.Second,
	}

	for _, cfg := range configs {
		discoveryURL := fmt.Sprintf("%s/.well-known/openid-configuration", cfg.IssuerURL)

		// Step 1: Fetch the discovery document
		resp, err := client.Get(discoveryURL)
		if err != nil {
			fmt.Printf("Error fetching discovery document for %s: %v\n", cfg.IssuerURL, err)
			continue
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			fmt.Printf("Unexpected status code from discovery document for %s: %d\n", cfg.IssuerURL, resp.StatusCode)
			continue
		}

		var discovery DiscoveryDoc
		if err := json.NewDecoder(resp.Body).Decode(&discovery); err != nil {
			fmt.Printf("Failed to parse discovery document for %s: %v\n", cfg.IssuerURL, err)
			continue
		}

		// Step 2: Fetch the JWKS
		resp, err = client.Get(discovery.JWKSURI)
		if err != nil {
			fmt.Printf("Error fetching JWKS for %s: %v\n", cfg.IssuerURL, err)
			continue
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			fmt.Printf("Unexpected status code from JWKS endpoint for %s: %d\n", cfg.IssuerURL, resp.StatusCode)
			continue
		}

		var jwks JWKS
		if err := json.NewDecoder(resp.Body).Decode(&jwks); err != nil {
			fmt.Printf("Failed to parse JWKS for %s: %v\n", cfg.IssuerURL, err)
			continue
		}

		mu.Lock()
		if _, ok := oidcPubKeys[cfg.IssuerURL]; !ok {
			oidcPubKeys[cfg.IssuerURL] = make(map[string][]JWK)
		}
		for _, key := range jwks.Keys {
			oidcPubKeys[cfg.IssuerURL][key.Alg] = append(oidcPubKeys[cfg.IssuerURL][key.Alg], key)
		}
		mu.Unlock()
	}
}

func InitAuth(oidc *[]config.OIDCConfig, path string) error {
    oidcPubKeys = make(map[string]map[string][]JWK)
	key, err := LoadJWTSigningKey(path)
    if err != nil {
        return fmt.Errorf("failed to load private key: %w", err)
    }

    block, _ := pem.Decode(key)
    if block == nil || block.Type != "PRIVATE KEY" {
        return fmt.Errorf("failed to decode private key, got block type: %v", block.Type)
    }

    parsedKey, err := x509.ParsePKCS8PrivateKey(block.Bytes)
    if err != nil {
        return fmt.Errorf("invalid PKCS#8 RSA private key: %w", err)
    }

    rsaKey, ok := parsedKey.(*rsa.PrivateKey)
    if !ok {
        return fmt.Errorf("parsed key is not an RSA private key")
    }

    // Store keys globally
    jwtSigningKey = rsaKey
    JWTPubKey = rsaKey.PublicKey

    // Add local public key to oidcPubKeys
    mu.Lock()
    if _, ok := oidcPubKeys[localIssuer]; !ok {
        oidcPubKeys[localIssuer] = make(map[string][]JWK)
    }

    // Encode modulus N and exponent E to base64url
    n := base64.RawURLEncoding.EncodeToString(rsaKey.PublicKey.N.Bytes())
    e := base64.RawURLEncoding.EncodeToString(big.NewInt(int64(rsaKey.PublicKey.E)).Bytes())

    localJWK := JWK{
        Kid: localKid,
        Alg: localAlg,
        Kty: "RSA",
        Use: "sig",
        N:   n,
        E:   e,
    }

    oidcPubKeys[localIssuer][localAlg] = append(oidcPubKeys[localIssuer][localAlg], localJWK)
    mu.Unlock()

    // Optionally: also call GetOIDCPubKeys for remote issuers
    if oidc != nil {
        GetOIDCPubKeys(*oidc)
    }

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

type StringList []string

func NewStringList(s string) StringList {
    return StringList{s}
}

func (s *StringList) UnmarshalJSON(data []byte) error {
    // Try to unmarshal as single string
    var single string
    if err := json.Unmarshal(data, &single); err == nil {
        *s = []string{single}
        return nil
    }

    // Else, try to unmarshal as []string
    var list []string
    if err := json.Unmarshal(data, &list); err == nil {
        *s = list
        return nil
    }

    // Neither worked; return error
    return fmt.Errorf("StringList: invalid JSON, expected string or []string: %s", string(data))
}

func (s *StringList) Contains(value string) bool {
    for _, v := range *s {
        if v == value {
            return true
        }
    }
    return false
}

type OIDCClaims struct {
    Sub     *uuid.UUID  `json:"sub"`   // Subject (user ID)
    Email   string  `json:"email"` // Optional additional claim
    Iss     string  `json:"iss"`
    Aud     StringList  `json:"aud"`
    jwt.RegisteredClaims        // includes exp, iat, etc.
}

func AuthenticateAndIssueToken(db *gorm.DB, email, password string) (string, error) {
    user, err := GetUserByEmail(db, email)
    if err != nil {
        return "", err
    }

    if user.PasswordHash == nil {
        return "", fmt.Errorf("no password found for user")
    }

    if err := bcrypt.CompareHashAndPassword([]byte(*user.PasswordHash), []byte(password)); err != nil {
        return "", errors.New("invalid password")
    }

    if user.Subject == nil {
        return "", fmt.Errorf("user subject is nil")
    }

    expiration := time.Now().Add(time.Hour)
    claims := OIDCClaims{
        Sub:   user.Subject, // *string
        Email: user.Email,
        Aud:   NewStringList("storyteller"),
        Iss:   localIssuer,
        RegisteredClaims: jwt.RegisteredClaims{
            ExpiresAt: jwt.NewNumericDate(expiration),
            IssuedAt:  jwt.NewNumericDate(time.Now()),
        },
    }

    // Create token with claims and explicitly set kid in header
    token := jwt.NewWithClaims(jwt.SigningMethodRS256, claims)
    token.Header["kid"] = localKid

    signedToken, err := token.SignedString(jwtSigningKey)
    if err != nil {
        return "", fmt.Errorf("failed to sign token: %w", err)
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

func FindPublicKey(issuer, kid, alg string) (*JWK, error) {
	mu.Lock()
	defer mu.Unlock()

	algMap, ok := oidcPubKeys[issuer]
	if !ok {
		return nil, fmt.Errorf("unknown issuer: %s", issuer)
	}
	keys, ok := algMap[alg]
	if !ok {
		return nil, fmt.Errorf("no keys for algorithm: %s", alg)
	}

	for _, key := range keys {
		if key.Kid == kid {
			return &key, nil
		}
	}
	return nil, fmt.Errorf("no key found for kid: %s", kid)
}

type TokenHeader struct {
	Alg string `json:"alg"`
	Kid string `json:"kid"`
}

// parseTokenHeader decodes the JWT header part
func parseTokenHeader(token string) (*TokenHeader, error) {
	parts := strings.Split(token, ".")
	if len(parts) < 2 {
		return nil, fmt.Errorf("invalid token format")
	}
	headerBytes, err := base64.RawURLEncoding.DecodeString(parts[0])
	if err != nil {
		return nil, fmt.Errorf("failed to decode token header: %v", err)
	}
	var header TokenHeader
	if err := json.Unmarshal(headerBytes, &header); err != nil {
		return nil, fmt.Errorf("failed to unmarshal token header: %v", err)
	}
	return &header, nil
}

type TokenPayload struct {
	Iss string `json:"iss"`
}

func parseTokenPayload(token string) (*TokenPayload, error) {
	parts := strings.Split(token, ".")
	if len(parts) < 2 {
		return nil, fmt.Errorf("invalid token format")
	}
	payloadBytes, err := base64.RawURLEncoding.DecodeString(parts[1])
	if err != nil {
		return nil, fmt.Errorf("failed to decode token payload: %v", err)
	}
	var payload TokenPayload
	if err := json.Unmarshal(payloadBytes, &payload); err != nil {
		return nil, fmt.Errorf("failed to unmarshal token payload: %v", err)
	}
	return &payload, nil
}

func ValidateToken(tokenString string) (*OIDCClaims, error) {
	// Step 1: Parse header to get kid and alg
	header, err := parseTokenHeader(tokenString)
	if err != nil {
		return nil, err
	}

	// Step 2: Parse payload to get iss
	payload, err := parseTokenPayload(tokenString)
	if err != nil {
		return nil, err
	}
	issuer := payload.Iss

	// Step 3: Find the matching JWK
	key, err := FindPublicKey(issuer, header.Kid, header.Alg)
	if err != nil {
		return nil, err
	}

	// Step 4: Build rsa.PublicKey from JWK
	pubKey, err := BuildRSAPublicKey(key.N, key.E)
	if err != nil {
		return nil, err
	}

	// Step 5: Parse and validate the token
	token, err := jwt.ParseWithClaims(tokenString, &OIDCClaims{}, func(token *jwt.Token) (any, error) {
		// Optional: check alg matches expected
		if alg, ok := token.Header["alg"].(string); !ok || alg != key.Alg {
			return nil, fmt.Errorf("unexpected signing algorithm: got %v, expected %v", alg, key.Alg)
		}
		return pubKey, nil
	})
	if err != nil {
		return nil, fmt.Errorf("token parsing failed: %w", err)
	}

	claims, ok := token.Claims.(*OIDCClaims)
	if !ok || !token.Valid {
		return nil, errors.New("invalid token or claims")
	}

	// Step 6: Additional checks (issuer, audience, expiration, etc.)
	// adjust as needed
	if claims.Iss != issuer {
		return nil, fmt.Errorf("invalid issuer: %s", claims.Iss)
	}
	if !claims.Aud.Contains("storyteller") {
		return nil, fmt.Errorf("invalid audience: %s", claims.Aud)
	}
	if claims.ExpiresAt != nil && claims.ExpiresAt.Time.Before(time.Now()) {
		return nil, errors.New("token is expired")
	}

	return claims, nil
}

// JWTMiddleware validates the JWT and injects claims into the Gin context
func JWTMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Get the Authorization header
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" || !strings.HasPrefix(authHeader, "Bearer ") {
			fmt.Printf("failed to get auth header")
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "missing or malformed Authorization header"})
			return
		}


		// Extract token string
		tokenString := strings.TrimPrefix(authHeader, "Bearer ")

		// Validate token
		claims, err := ValidateToken(tokenString)
		if err != nil {
			fmt.Printf("failed to validate token: %s", err)
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": err})
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