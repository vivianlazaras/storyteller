package handlers

import (
	"net/http"
	"time"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/auth"
	"gorm.io/gorm"
	
)

func RegisterLocationRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/locations/", auth.JWTMiddleware(), GetLocations)
	r.GET("/locations/:id", auth.JWTMiddleware(), GetLocation)
	r.POST("/locations/", auth.JWTMiddleware(), CreateLocation)
	r.GET("/locations/filter", auth.JWTMiddleware(), GetLocationsByStory)
	return r
}

type LocationBuilder struct {
	ID				uuid.UUID		`json:"id"`
	Name			string			`json:"name"`
	Description		*string			`json:"description"`
	Tags			[]string		`json:"tags"`
	Thumbnail		*ImageBuilder	`json:"thumbnail"`
}

type LocationRender struct {
	ID				uuid.UUID	`json:"id"`
	Name			string	`json:"name"`
	Description		*string	`json:"description"`
	Thumbnail		*model.Image	`json:"thumbnail"`
	Images			[]model.Image	`json:"images"`
	Tags			[]model.Tag		`json:"tags"`
	Created			int64			`json:"created"`
}

func GetLocations(c *gin.Context) {
	user, uerr := auth.GetUserFromClaims(db.DB, c)
	if uerr != nil {
		c.JSON(http.StatusUnauthorized, uerr)
		return
	}
	// grab all stories where public = true
	var locations []model.Location
	var renders []LocationRender
    err := db.DB.
        Where("entities.active = ?", true).
        Joins("JOIN entities ON entities.id = locations.id").
        Find(&locations).Error
	
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}

	for _, location := range locations {
		render, renderr := RenderLocation(db.DB, location, user.ID)
		if renderr != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": renderr})
			return
		}
		renders = append(renders, render)
		
	}
	c.JSON(http.StatusOK, renders)
}

func RenderLocation(tx *gorm.DB, location model.Location, userID uuid.UUID) (LocationRender, error) {

	var locid = location.ID
	var thumbnail *model.Image = nil
	tags, tagerr := SelectTagsByEntityID(locid)
	if tagerr != nil {
		return LocationRender{}, nil
	}
	
	if location.Thumbnail != nil {
		image, thmerr := GetByID[model.Image](db.DB, "images", *location.Thumbnail, userID);
		if thmerr != nil {
			return LocationRender{}, thmerr
		}
		thumbnail = image
	}

	images, imgerr := GetImagesByParentID(tx, locid)
	if imgerr != nil {
		return LocationRender{}, imgerr
	}

	return LocationRender{
		Thumbnail: thumbnail,
		Tags:  tags,
		ID: locid,
		Name: location.Name,
		Description: location.Description,
		Images: images,
		Created: location.Created,
	}, nil
}

func GetLocation(c *gin.Context) {
	user, uerr := auth.GetUserFromClaims(db.DB, c)
	if uerr != nil {
		c.JSON(http.StatusUnauthorized, uerr)
		return
	}

	idParam := c.Param("id")

	// Validate UUID
	locationID, err := uuid.Parse(idParam)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid UUID format"})
		return
	}

	var location model.Location
	if err := db.DB.First(&location, "id = ?", locationID).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "location not found"})
		return
	}

	render, rendererr := RenderLocation(db.DB, location, user.ID)
	if rendererr != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": rendererr})
		return
	}
	// Return result
	c.JSON(http.StatusOK, render)
}

func CreateNewLocation(tx *gorm.DB, builder LocationBuilder, userID, groupID uuid.UUID) (model.Location, error) {
	now := time.Now().Unix()

	var location = model.Location {
		ID: uuid.New(),
		Name: builder.Name,
		Description: builder.Description,
		Created: now,
		LastEdited: &now,
	}

	err := tx.Create(&location).Error;
	if err != nil {
		//tx.Rollback()
		fmt.Printf("location error: %s", err)
		return model.Location{}, err
	}

	// this will also check to ensure the user has access to the group, so that logic is in one place
	if err := CreateNewEntity(tx, location.ID, userID, groupID); err != nil {
		return model.Location{}, err
	}

	if builder.Thumbnail != nil {
		builder.Thumbnail.Parent = &location.ID
		images, imgerr := CreateNewImage(tx, *builder.Thumbnail, userID, groupID)
		if imgerr != nil {
			fmt.Printf("image error: %s", imgerr)
			//tx.Rollback()
			return model.Location{}, imgerr
		}
		if len(images) > 0 {
			location.Thumbnail = &images[0].ID
			thumbID := images[0].ID
			updateErr := tx.Model(&location).Update("thumbnail", thumbID).Error
			if updateErr != nil {
				fmt.Printf("thumbnail update error: %s", updateErr)
				return model.Location{}, updateErr
			}

			// Reflect change in the return value
			location.Thumbnail = &thumbID
		}
	}

	tagerr := InsertTagsForEntity(tx, location.ID, builder.Tags)
	if tagerr != nil {
		//tx.Rollback()
		fmt.Printf("tagerr %s", tagerr)
		return model.Location{}, tagerr
	}
	//tx.Commit()
	return location, nil
}

func CreateLocation(c *gin.Context) {
	user, uerr := auth.GetUserFromClaims(db.DB, c)
	if uerr != nil {
		c.JSON(http.StatusUnauthorized, gin.H{ "error": uerr})
		return
	}
	var builder LocationBuilder
	if err := c.ShouldBindJSON(&builder); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": "invalid request: " + err.Error(),
		})
		return
	}

	tx := db.DB.Begin()
	if tx.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to create transaction"})
		return
	}

	location, dberr := CreateNewLocation(tx, builder, user.ID, user.DefaultGroup);
	if dberr != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{"error": dberr})
		return
	}

	// Update group_id in entities table for the created story
	if err := tx.Model(&model.Entity{}).
		Where("id = ?", location.ID).
		Update("group_id", user.DefaultGroup).Error; err != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to update group_id: " + err.Error()})
		return
	}

	// Commit transaction
	if err := tx.Commit().Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to commit transaction"})
		return
	}
	c.JSON(http.StatusOK, location)
}

func GetLocationsByStory(c *gin.Context) {
	IDString := c.Query("parent")
	parentID, iderr := uuid.Parse(IDString)
	if iderr != nil {
		fmt.Printf("failed to parse UUID: %s", IDString)
		c.JSON(http.StatusBadRequest, gin.H{"error": "failed to parse parent as UUID"})
		return
	}

	var locations []model.Location
	err := db.DB.
		Model(&model.Location{}).
		Joins("JOIN relations ON relations.child = locations.id").
		Where("relations.parent = ? AND relations.parent_category = ? AND relations.child_category = ?", parentID, "stories", "locations").
		Find(&locations).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	locationRenders := make([]LocationRender, 0, len(locations))

	for _, loc := range locations {
		var thumbnail *model.Image
		if loc.Thumbnail != nil {
			var img model.Image
			if err := db.DB.First(&img, "id = ?", loc.Thumbnail).Error; err == nil {
				thumbnail = &img
			} else {
				fmt.Printf("warning: failed to load thumbnail for location %s: %v\n", loc.ID, err)
			}
		}

		render := LocationRender{
			ID:          loc.ID,
			Name:        loc.Name,
			Description: loc.Description,
			Thumbnail:   thumbnail,
		}
		locationRenders = append(locationRenders, render)
	}

	c.JSON(http.StatusOK, locationRenders)
}