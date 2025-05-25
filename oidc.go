package middleware

import (
	"context"
	"net/http"
	"strings"

	"github.com/coreos/go-oidc"
	"github.com/gin-gonic/gin"
)

type Config struct {
	Issuer           string
	ClientID         string
	ClientSecret     string
	AutoCreateUser   bool
}

var CONFIG *Config;

type OIDCMiddleware struct {
	verifier *oidc.IDTokenVerifier
}

func New(config *Config) (*OIDCMiddleware, error) {
	provider, err := oidc.NewProvider(context.Background(), config.Issuer)
	CONFIG = config
	if err != nil {
		return nil, err
	}

	verifier := provider.Verifier(&oidc.Config{
		ClientID: config.ClientID,
	})

	return &OIDCMiddleware{verifier: verifier}, nil
}

// RequireAuth returns a Gin middleware function that validates the access token
func (m *OIDCMiddleware) RequireAuth() gin.HandlerFunc {
	return func(c *gin.Context) {
		authHeader := c.GetHeader("Authorization")
		if !strings.HasPrefix(authHeader, "Bearer ") {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "missing or invalid token"})
			return
		}

		rawToken := strings.TrimPrefix(authHeader, "Bearer ")

		idToken, err := m.verifier.Verify(c.Request.Context(), rawToken)
		if err != nil {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "invalid token", "details": err.Error()})
			return
		}

		var claims map[string]interface{}
		if err := idToken.Claims(&claims); err != nil {
			c.AbortWithStatusJSON(http.StatusInternalServerError, gin.H{"error": "failed to parse claims"})
			return
		}

		// Store both token and claims in context
		c.Set("token", idToken)
		c.Set("claims", claims)

		c.Next()
	}
}