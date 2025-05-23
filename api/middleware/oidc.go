package middleware

import (
	"net/http"
	"context"
	"strings"
	
	"github.com/coreos/go-oidc"
	"github.com/gin-gonic/gin"
	
)

type Config struct {
	issuer string
	AutoCreateUser bool
	clientID string
	clientSecret string
}

func defaultConfig(clientSecret string) Config {
	return Config {
		issuer: "https://localhost/realms/master",
		AutoCreateUser: true,
		clientID: "storyteller",
		clientSecret: clientSecret,
	}
}

var verifier *oidc.IDTokenVerifier
var CONFIG *Config

func initOIDC(config *Config) {
	provider, err := oidc.NewProvider(context.Background(), config.issuer)
	if err != nil {
		panic(err)
	}

	verifier = provider.Verifier(&oidc.Config{
		ClientID: config.clientID, // same as what Rust used to get token
	})
}

func RequireOIDC() gin.HandlerFunc {
	return func(c *gin.Context) {
		authHeader := c.GetHeader("Authorization")
		if !strings.HasPrefix(authHeader, "Bearer ") {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "missing or invalid token"})
			return
		}

		token := strings.TrimPrefix(authHeader, "Bearer ")

		idToken, err := verifier.Verify(c.Request.Context(), token)
		if err != nil {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{"error": "invalid token"})
			return
		}

		var claims map[string]interface{}
		if err := idToken.Claims(&claims); err != nil {
			c.AbortWithStatusJSON(http.StatusInternalServerError, gin.H{"error": "failed to parse claims"})
			return
		}

		c.Set("claims", claims)
		c.Next()
	}
}