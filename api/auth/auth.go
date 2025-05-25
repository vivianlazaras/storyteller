package auth

import (
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/golang-jwt/jwt/v4"
	"github.com/MicahParks/keyfunc"
)

var jwks *keyfunc.JWKS

// InitJWKS initializes and caches the JWKS from Keycloak
func InitJWKS(realmURL string) error {
	jwksURL := fmt.Sprintf("%s/protocol/openid-connect/certs", realmURL)

	options := keyfunc.Options{
		RefreshInterval:   time.Hour,
		RefreshRateLimit:  time.Minute * 5,
		RefreshTimeout:    time.Second * 10,
		RefreshUnknownKID: true,
	}

	var err error
	jwks, err = keyfunc.Get(jwksURL, options)
	return err
}

// JWTMiddleware returns a Gin middleware that verifies the JWT
func JWTMiddleware(expectedAudience string) gin.HandlerFunc {
	return func(c *gin.Context) {
		authHeader := c.GetHeader("Authorization")
		if authHeader == "" || !strings.HasPrefix(authHeader, "Bearer ") {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "missing or invalid Authorization header"})
			return
		}

		tokenString := strings.TrimPrefix(authHeader, "Bearer ")

		// Parse and validate the token
		token, err := jwt.Parse(tokenString, jwks.Keyfunc)
		if err != nil || !token.Valid {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "invalid token"})
			return
		}

		claims, ok := token.Claims.(jwt.MapClaims)
		if !ok {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "invalid token claims"})
			return
		}

		// Optional: verify audience
		if aud, ok := claims["aud"].(string); !ok || aud != expectedAudience {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{"error": "invalid audience"})
			return
		}

		// Add claims to context
		c.Set("claims", claims)

		c.Next()
	}
}

// ExtractClaim gets a claim from the Gin context
func ExtractClaim(c *gin.Context, key string) (string, error) {
	claimsAny, exists := c.Get("claims")
	if !exists {
		return "", errors.New("no claims in context")
	}
	claims, ok := claimsAny.(jwt.MapClaims)
	if !ok {
		return "", errors.New("invalid claims type")
	}
	val, ok := claims[key].(string)
	if !ok {
		return "", fmt.Errorf("claim %q not found or not a string", key)
	}
	return val, nil
}
