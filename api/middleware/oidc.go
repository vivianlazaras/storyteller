package middleware

import (
	"net/http"
	"context"
	"strings"
	
	"github.com/coreos/go-oidc"
	"github.com/gin-gonic/gin"
	
)

var verifier *oidc.IDTokenVerifier

func initOIDC() {
	provider, err := oidc.NewProvider(context.Background(), "https://your-keycloak-domain/auth/realms/your-realm")
	if err != nil {
		panic(err)
	}

	verifier = provider.Verifier(&oidc.Config{
		ClientID: "your-client-id", // same as what Rust used to get token
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