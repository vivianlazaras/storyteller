package handlers

import (
	"net/http"
	"time"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

type FragmentBuilder struct {
	Parent		*string			`json:"parent"`
	Category	*string			`json:"category"`
	Content     string          `json:"content"`
	Name		string			`json:"name"`
	Tags		[]string		`json:"tags"`
}

type FragmentUpdate struct {
	ID			uuid.UUID	`json:"id"`
	Name		string		`json:"name"`
	Content		string		`json:"content"`
}

type FragmentRender struct {
	ID		uuid.UUID	`json:"id"`
	Name	string		`json:"name"`
	Description	string	`json:"description"`
	Content	string		`json:"content"`
	Images	[]model.Image `json:"images"`
	Created		string	`json:"created"`
	LastEdited	string	`json:"last_edited"`
}

func RegisterFragmentRoutes(r *gin.Engine) *gin.Engine {
	
	r.GET("/fragments", auth.JWTMiddleware(), GetFragmentsByEntity)
	r.GET("/fragments/:id", auth.JWTMiddleware(), GetFragmentById)
	r.POST("/fragments/", auth.JWTMiddleware(), CreateFragment)
	r.GET("/fragments/", auth.JWTMiddleware(), GetFragments)
	return r
}

func GetFragments(c *gin.Context) {
	var fragments []model.Fragment
	err := db.DB.
		Where("entities.active = ?", true).
		Joins("JOIN entities ON entities.id = fragments.id").
		Find(&fragments).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}
	c.JSON(http.StatusOK, fragments)
}

func linkFragment(tx *gorm.DB, fragment *FragmentBuilder, id uuid.UUID) error {
	if fragment.Category != nil && fragment.Parent != nil {
		relation := model.Relation{
			Parent:         *fragment.Parent,
			Child:          id.String(),
			ParentCategory: *fragment.Category,
			ChildCategory:  "fragments", // assuming child is always a fragment
			Description:    "",        // or you can add logic to populate this if needed
		}

		if err := tx.Create(&relation).Error; err != nil {
			return fmt.Errorf("failed to create relation: %w", err)
		}
	}
	return nil
}

func GetFragmentsByEntity(c *gin.Context) {
	IDString := c.Query("parent")
	parentID, iderr := uuid.Parse(IDString)
	if iderr != nil {
		fmt.Printf("failed to parse UUID: %s", IDString)
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse story as UUID"})
		return
	}

	fragments,err := selectFragmentsByEntity(db.DB, parentID)

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "db error"})
		return
	}

	c.JSON(http.StatusOK, fragments)
}

/*
func selectFragmentsByStory(db *gorm.DB, parentID uuid.UUID) ([]model.Fragment, error) {

	var fragments []model.Fragment
	err := db.
		Model(&model.Fragment{}).
		Joins("JOIN relations ON relations.child = fragments.id").
		Where("relations.parent = ? AND relations.parent_category = ? AND relations.child_category = ?", parentID, "stories", "fragments").
		Order("fragments.last_edited ASC").
		Find(&fragments).Error

	return fragments, err
}*/

func selectFragmentsByEntity(db *gorm.DB, parentID uuid.UUID) ([]FragmentRender, error) {
	var fragments []model.Fragment

	err := db.
		Model(&model.Fragment{}).
		Joins("JOIN relations ON relations.child = fragments.id").
		Where("relations.parent = ? AND relations.child_category = ?", parentID, "fragments").
		Order("fragments.last_edited ASC").
		Find(&fragments).Error

	if err != nil {
		return nil, err
	}

	var results []FragmentRender
	for _, frag := range fragments {
		var images []model.Image
		err := db.
			Model(&model.Image{}).
			Joins("JOIN relations ON relations.child = images.id").
			Where("relations.parent = ? AND relations.parent_category = ? AND relations.child_category = ?", frag.ID, "fragments", "images").
			Find(&images).Error
		if err != nil {
			return nil, err
		}

		render := FragmentRender{
			ID:          uuid.MustParse(frag.ID),
			Name:        frag.Name,
			Description: frag.Description,
			Content:     frag.Content,
			Images:      images,
			Created:     time.Unix(int64(frag.Created), 0).UTC().Format(time.RFC3339),
			LastEdited:  time.Unix(int64(frag.LastEdited), 0).UTC().Format(time.RFC3339),
		}
		results = append(results, render)
	}

	return results, nil
}

func GetFragmentById(c *gin.Context) {
	fragment, err := db.GetByCtxID[model.Fragment](c, "fragments");
	if err != nil {
		return
	}

	// get fragments, characters, places
	c.JSON(http.StatusOK, fragment)
}

func CreateNewFragment(tx *gorm.DB, fragment *FragmentBuilder, creatorID uuid.UUID) (model.Fragment, error) {
	now := time.Now().Unix()
	fragmentid := uuid.New()
	

	metadata, err := createDefaultMetadata(creatorID)
	if err != nil {
		return model.Fragment{}, err
	}

	var newfragment = model.Fragment {
		ID: 		fragmentid.String(),
		Metadata:	metadata.ID,
		Content:	fragment.Content,
		Name:		fragment.Name,
		LastEdited:	now,
		Created:	now,
	}

	if fragment.Name != "" {
		fragmentdberr := db.DB.Create(&newfragment).Error
		if fragmentdberr != nil {
			return newfragment, fragmentdberr
		}
	}

	tagerr := InsertTagsForEntity(tx, fragmentid, fragment.Tags)
	linkerr := linkFragment(tx, fragment, fragmentid)
	
	if tagerr != nil {
		return newfragment, tagerr
	}
	return newfragment, linkerr
}

func CreateFragment(c *gin.Context) {
	// I do need to handle automatic user creation if user not found
	// aka handle settings
	// also this is the best place to try and update timeline (if it exists)
	user, err := auth.GetUserFromClaims(db.DB, c)

	var fragment FragmentBuilder
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

	tx := db.DB.Begin()
	if tx.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to create transaction"})
		return
	}
	newfragment, newerr := CreateNewFragment(tx, &fragment, parsedUUID)
	if newerr != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{ "error": "unkown panic" })
		return
	}
	tx.Commit()
	c.JSON(http.StatusOK, newfragment)
}

func UpdateFragment(db *gorm.DB, update FragmentUpdate, userID string) error {
	
	tx := db.Begin()
	if tx.Error != nil {
		return tx.Error
	}

	// Step 1: Load current fragment
	var fragment model.Fragment
	if err := tx.First(&fragment, "id = ?", update.ID).Error; err != nil {
		tx.Rollback()
		return err
	}

	// Step 2: Compute line diff
	diff := CalcDiff(&fragment.Content, &update.Content)
	if len(diff) == 0 {
		tx.Commit()
		return nil // No changes
	}

	// Step 3: Create a group ID to associate these edits
	groupID := uuid.New().String()
	now := time.Now().Unix()

	// Step 4: Create one Edit per change
	for _, line := range diff {
		edit := model.Edit{
			ID:        uuid.New().String(),
			UpdateID:   groupID,
			Date:      now,
			Editor:    userID,
			Value:     line.Value,
			Prevvalue: line.Previous,
			Entity:    fragment.ID,
			Field:     "content",
			Change:    line.Change,
		}
		if err := tx.Create(&edit).Error; err != nil {
			tx.Rollback()
			return err
		}
	}

	// Step 5: Update the fragment content
	fragment.Content = update.Content
	if err := tx.Save(&fragment).Error; err != nil {
		tx.Rollback()
		return err
	}

	return tx.Commit().Error
}

func EditFragment(c *gin.Context) {
	user, err := auth.GetUserFromClaims(db.DB, c)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "failed to get user from claims shomewhow"})
		return
	}
	var fragmentUpdate FragmentUpdate
	if err := c.ShouldBindJSON(&fragmentUpdate); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to bind JSON"})
		return
	}

	updateErr := UpdateFragment(db.DB, fragmentUpdate, user.ID)
	if updateErr != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to update fragment"})
		return
	}

	c.JSON(http.StatusOK, fragmentUpdate)
}