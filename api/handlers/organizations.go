package handlers
import (
	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
	"net/http"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/model"
)

func RegisterOrgRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/", ListOrganizations);
	r.GET("/:id", GetOrganization);
	r.POST("/", CreateOrganization);
	return r;
}

type OrgBuilder struct {
	Name	string	`json:"name"`
	Description	*string	`json:"description"`
}

func CreateNewOrganization(tx *gorm.DB, builder OrgBuilder) (model.Organization, error) {
	return model.Organization{}, nil
}

func GetOrganization(c *gin.Context) {}
func ListOrganizations(c *gin.Context) {}
func CreateOrganization(c *gin.Context) {
	
	tx := db.DB.Begin()
	// failed to begin database transaction, internal server error
	/*if err != nil {
		c.JSON(http.StatusInternalServerError, err)
		return
	}*/

	_, uerr := auth.GetUserFromClaims(db.DB, c);
	// failed to grab user data from API key, assume unauthorized
	if uerr != nil {
		c.JSON(http.StatusUnauthorized, uerr)
		return
	}
	var builder = OrgBuilder{}
	CreateNewOrganization(tx, builder)
	
}
