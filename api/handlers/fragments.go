package handlers

import (
	"net/http"
	"time"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	// "github.com/vivianlazaras/storyteller/auth"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
)

type CreateStoryFragment struct {
	Parent		*string			`json:"parent"`
	Category	*string			`json:"category"`
	Content     string          `json:"content"`
	Name		string			`json:"name"`
	Tags		[]string		`json:"tags"`
	Image		*string			`json:"image"`
}

func RegisterFragmentRoutes(r *gin.Engine) *gin.Engine {
	
	r.GET("/fragments", GetFragmentsByStory)
	r.GET("/fragments/:id", GetFragmentById)
	r.POST("/fragments/", CreateFragment)
	return r
}

func linkFragment(fragment *CreateStoryFragment, id uuid.UUID) error {
	if fragment.Category != nil && fragment.Parent != nil {
		relation := model.Relation{
			Parent:         *fragment.Parent,
			Child:          id.String(),
			ParentCategory: *fragment.Category,
			ChildCategory:  "fragments", // assuming child is always a fragment
			Description:    "",        // or you can add logic to populate this if needed
		}

		if err := db.DB.Create(&relation).Error; err != nil {
			return fmt.Errorf("failed to create relation: %w", err)
		}
	}
	return nil
}

func GetFragmentsByStory(c *gin.Context) {
	IDString := c.Query("parent")
	parentID, iderr := uuid.Parse(IDString)
	if iderr != nil {
		fmt.Printf("failed to parse UUID: %s", IDString)
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse story as UUID"})
		return
	}

	var fragments []model.Fragment
	err := db.DB.
		Model(&model.Fragment{}).
		Joins("JOIN relations ON relations.child = fragments.id").
		Where("relations.parent = ? AND relations.parent_category = ? AND relations.child_category = ?", parentID, "stories", "fragments").
		Order("fragments.last_edited ASC").
		Find(&fragments).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "db error"})
		return
	}

	c.JSON(http.StatusOK, fragments)
}

func GetFragmentById(c *gin.Context) {
	fragment, err := db.GetByCtxID[model.Fragment](c, "fragments");
	if err != nil {
		return
	}

	// get fragments, characters, places
	c.JSON(http.StatusOK, fragment)
}

func CreateNewFragment(fragment *CreateStoryFragment, creatorID uuid.UUID) (model.Fragment, error) {
	now := time.Now().Unix()
	image		:= ""
	fragmentid := uuid.New()
	if fragment.Image != nil {
		image = *fragment.Image
	}

	metadata, err := createDefaultMetadata(creatorID)
	if err != nil {
		return model.Fragment{}, err
	}

	var newfragment = model.Fragment {
		ID: 		fragmentid.String(),
		Metadata:	metadata.ID,
		Content:	fragment.Content,
		Name:		fragment.Name,
		Image:		image,
		LastEdited:	now,
		Created:	now,
	}

	if fragment.Name != "" {
		fragmentdberr := db.DB.Create(&newfragment).Error
		if fragmentdberr != nil {
			return newfragment, fragmentdberr
		}
	}

	tagerr := InsertTagsForEntity(fragmentid, fragment.Tags)
	linkerr := linkFragment(fragment, fragmentid)
	
	if tagerr != nil {
		return newfragment, tagerr
	}
	return newfragment, linkerr
}

func CreateFragment(c *gin.Context) {
	// I do need to handle automatic user creation if user not found
	// aka handle settings
	user, err := getUserByEmail("vivianlazaras@gmail.com")

	var fragment CreateStoryFragment
	if err := c.ShouldBindJSON(&fragment); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "Invalid request: " + err.Error(),
		})
		return
	}

	parsedUUID, err := uuid.Parse(user.ID)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{ "error": "Internal Server Error: " + err.Error() })
		return
	}
	newfragment, newerr := CreateNewFragment(&fragment, parsedUUID)
	if newerr != nil {
		c.JSON(http.StatusInternalServerError, gin.H{ "error": "unkown panic" })
		return
	}

	c.JSON(http.StatusOK, newfragment)
}