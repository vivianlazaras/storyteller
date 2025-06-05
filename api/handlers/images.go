package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/model"
)

func RegisterImageRoutes(r *gin.Engine) *gin.Engine {
	r.POST("/images/", CreateImage)
	return r
}

type ExifTag struct {
	Tag   int32    `json:"tag"`
	Value string `json:"value"`
}

func ExifTagName(code int) string {
	tagNames := map[int]string{
		0x010F: "Make",
		0x0110: "Model",
		0x0112: "Orientation",
		0x0131: "Software",
		0x0132: "DateTime",
		0x829A: "ExposureTime",
		0x829D: "FNumber",
		0x8827: "ISOSpeedRatings",
		0x9003: "DateTimeOriginal",
		0x9004: "DateTimeDigitized",
		0x9201: "ShutterSpeedValue",
		0x9202: "ApertureValue",
		0x9204: "ExposureBiasValue",
		0x9207: "MeteringMode",
		0x9209: "Flash",
		0xA002: "PixelXDimension",
		0xA003: "PixelYDimension",
		0xA405: "FocalLengthIn35mmFilm",

		// GPS Tags
		0x0000: "GPSVersionID",
		0x0001: "GPSLatitudeRef",
		0x0002: "GPSLatitude",
		0x0003: "GPSLongitudeRef",
		0x0004: "GPSLongitude",
		0x0005: "GPSAltitudeRef",
		0x0006: "GPSAltitude",
		0x0010: "GPSImgDirectionRef",
		0x0011: "GPSImgDirection",
	}

	if name, ok := tagNames[code]; ok {
		return name
	}
	return "UnknownTag"
}

type ImageBuilder struct {
	Url         string     `json:"url"`
	Description *string    `json:"description"`
	Tags        []string   `json:"tags"`
	ExifTags    []ExifTag  `json:"exif_tags"`
	Parent      uuid.UUID  `json:"parent"`
	Category    string     `json:"category"`
}

func CreateNewImage(builder ImageBuilder) (model.Image, error) {
	var description = ""
	if builder.Description != nil {
		description = *builder.Description
	}

	image := model.Image{
		ID:          uuid.New().String(),
		URL:         builder.Url,
		Description: description,
	}

	tx := db.DB.Begin()

	if err := tx.Create(&image).Error; err != nil {
		tx.Rollback()
		return model.Image{}, err
	}

	// Save EXIF tags
	for _, tag := range builder.ExifTags {
		exif := model.ExifTag{
			Image: image.ID,
			Tag:     tag.Tag,
			Value:   tag.Value,
		}
		if err := tx.Create(&exif).Error; err != nil {
			tx.Rollback()
			return model.Image{}, err
		}
	}

	// Save relations from image to parent
	relation := model.Relation{
		Parent:         builder.Parent.String(),
		Child:          image.ID,
		ParentCategory: builder.Category,
		ChildCategory:  "images",
	}
	if err := tx.Create(&relation).Error; err != nil {
		tx.Rollback()
		return model.Image{}, err
	}

	// Save tags (relations to tag table entries)
	for _, tagValue := range builder.Tags {
		tag := model.Tag{
			ID:     uuid.New().String(),
			Value:  tagValue,
			Entity: image.ID,
		}
		if err := tx.Create(&tag).Error; err != nil {
			tx.Rollback()
			return model.Image{}, err
		}
	}

	tx.Commit()
	return image, nil
}

func CreateImage(c *gin.Context) {
	var builder ImageBuilder
	if err := c.ShouldBindJSON(&builder); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid JSON: " + err.Error()})
		return
	}

	image, err := CreateNewImage(builder)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create image: " + err.Error()})
		return
	}

	c.JSON(http.StatusCreated, image)
}

func GetImagesByParent(c *gin.Context) {
	parentIDStr := c.Query("parent")
	parentCategory := c.Query("category")

	parentID, err := uuid.Parse(parentIDStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid parent UUID"})
		return
	}

	var images []model.Image

	err = db.DB.
		Table("images").
		Joins("JOIN relations ON relations.child = images.uuid").
		Where("relations.parent = ? AND relations.parent_category = ? AND relations.child_category = ?", parentID, parentCategory, "images").
		Find(&images).Error

	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to query images"})
		return
	}

	c.JSON(http.StatusOK, images)
}