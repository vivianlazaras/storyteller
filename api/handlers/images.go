package handlers

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/vivianlazaras/storyteller/model"
	"gorm.io/gorm"
	"fmt"
)

func RegisterImageRoutes(r *gin.Engine) *gin.Engine {
	r.POST("/assets/images/", auth.JWTMiddleware(), CreateImage)
	r.GET("/assets/images/:id", auth.JWTMiddleware(), GetImage)
	r.GET("/assets/images/", auth.JWTMiddleware(), ListImages)
	return r
}

type ExifTag struct {
	Tag   int32    `json:"tag"`
	Value string `json:"value"`
}

func ExifTagName(code int) string {
    tagNames := map[int]string{
        // TIFF/Image IFD tags
        0x0100: "ImageWidth",
        0x0101: "ImageLength",
        0x0102: "BitsPerSample",
        0x0103: "Compression",
        0x0106: "PhotometricInterpretation",
        0x010F: "Make",
        0x0110: "Model",
        0x0111: "StripOffsets",
        0x0112: "Orientation",
        0x0115: "SamplesPerPixel",
        0x0116: "RowsPerStrip",
        0x0117: "StripByteCounts",
        0x011A: "XResolution",
        0x011B: "YResolution",
        0x011C: "PlanarConfiguration",
        0x0128: "ResolutionUnit",
        0x0131: "Software",
        0x0132: "DateTime",
        0x013E: "WhitePoint",
        0x013F: "PrimaryChromaticities",
        0x0211: "YCbCrCoefficients",
        0x0212: "YCbCrSubSampling",
        0x0213: "YCbCrPositioning",
        0x0214: "ReferenceBlackWhite",
        0x8298: "Copyright",

        // EXIF IFD tags
        0x829A: "ExposureTime",
        0x829D: "FNumber",
        0x8822: "ExposureProgram",
        0x8827: "ISOSpeedRatings",
        0x8828: "OECF",
        0x9000: "ExifVersion",
        0x9003: "DateTimeOriginal",
        0x9004: "DateTimeDigitized",
        0x9101: "ComponentsConfiguration",
        0x9102: "CompressedBitsPerPixel",
        0x9201: "ShutterSpeedValue",
        0x9202: "ApertureValue",
        0x9203: "BrightnessValue",
        0x9204: "ExposureBiasValue",
        0x9205: "MaxApertureValue",
        0x9206: "SubjectDistance",
        0x9207: "MeteringMode",
        0x9208: "LightSource",
        0x9209: "Flash",
        0x920A: "FocalLength",
        0x927C: "MakerNote",
        0x9290: "UserComment",
        0xA000: "FlashpixVersion",
        0xA001: "ColorSpace",
        0xA002: "PixelXDimension",
        0xA003: "PixelYDimension",
        0xA004: "RelatedSoundFile",
        0xA005: "InteroperabilityIFDPointer",
        0xA20B: "FlashEnergy",
        0xA20C: "SpatialFrequencyResponse",
        0xA20E: "FocalPlaneXResolution",
        0xA20F: "FocalPlaneYResolution",
        0xA210: "FocalPlaneResolutionUnit",
        0xA214: "SubjectLocation",
        0xA215: "ExposureIndex",
        0xA217: "SensingMethod",
        0xA300: "FileSource",
        0xA301: "SceneType",
        0xA302: "CFAPattern",
        0xA403: "WhiteBalance",
        0xA405: "FocalLengthIn35mmFilm",
        0xA406: "SceneCaptureType",
        0xA407: "GainControl",
        0xA408: "Contrast",
        0xA409: "Saturation",
        0xA40A: "Sharpness",
        0xA40B: "DeviceSettingDescription",
        0xA40C: "SubjectDistanceRange",

        // GPS IFD tags
        0x0000: "GPSVersionID",
        0x0001: "GPSLatitudeRef",
        0x0002: "GPSLatitude",
        0x0003: "GPSLongitudeRef",
        0x0004: "GPSLongitude",
        0x0005: "GPSAltitudeRef",
        0x0006: "GPSAltitude",
        0x0007: "GPSTimeStamp",
        0x0008: "GPSSatellites",
        0x0009: "GPSStatus",
        0x000A: "GPSMeasureMode",
        0x000B: "GPSDOP",
        0x000C: "GPSSpeedRef",
        0x000D: "GPSSpeed",
        0x000E: "GPSTrackRef",
        0x000F: "GPSTrack",
        0x0010: "GPSImgDirectionRef",
        0x0011: "GPSImgDirection",
        0x0012: "GPSMapDatum",
        0x0013: "GPSDestLatitudeRef",
        0x0014: "GPSDestLatitude",
        0x0015: "GPSDestLongitudeRef",
        0x0016: "GPSDestLongitude",
        0x0017: "GPSDestBearingRef",
        0x0018: "GPSDestBearing",
        0x0019: "GPSDestDistanceRef",
        0x001A: "GPSDestDistance",
        0x001B: "GPSProcessingMethod",
        0x001C: "GPSAreaInformation",
        0x001D: "GPSDateStamp",
        0x001E: "GPSDifferential",

        // Interoperability / Composite
        0xA420: "ImageUniqueID",
        0xA432: "CameraOwnerName",
        0xA433: "BodySerialNumber",
    }

    if name, ok := tagNames[code]; ok {
        return name
    }
    return fmt.Sprintf("UnknownTag 0x%04X", code)
}

type ImageEntry struct {
	Url			string		`json:"url"`
	ExifTags	[]ExifTag	`json:"exif_tags"`
}

type ImageBuilder struct {
	Entries		[]ImageEntry	`json:"entries"`
	Description *string    	`json:"description"`
	Tags        []string   	`json:"tags"`
	Category    string     	`json:"category"`
	Parent		*uuid.UUID		`json:"parent"`
}

type ExifTagRender struct {
	Tag			string	`json:"tag"`
	Value		string	`json:"value"`
}

type ImageRender struct {
	ID		uuid.UUID		`json:"id"`
	Description	*string		`json:"description"`
	Tags		[]string	`json:"tags"`
	ExifTags	[]ExifTagRender	`json:"exif_tags"`
	Url			string			`json:"url"`
}

func CreateNewImage(tx *gorm.DB, builder ImageBuilder, userID, groupID uuid.UUID) ([]model.Image, error) {

	var images []model.Image
	
	for _, entry := range builder.Entries {
		image := model.Image{
			ID:          uuid.New(),
			URL:         entry.Url,
			Description: builder.Description,
		}

		if err := tx.Create(&image).Error; err != nil {
			return images, err
		}

		if err := CreateNewEntity(tx, image.ID, userID, groupID); err != nil {
			return images, err
		}

		// I want to create entities in bulk with a special function to avoid potentially costly permissions checks.
	
		// Save EXIF tags
		// exif tags should probably be behind group protections as well.
		for _, tag := range entry.ExifTags {
			exif := model.ExifTag{
				Image: image.ID,
				Tag:     tag.Tag,
				Value:   tag.Value,
			}
			if err := tx.Create(&exif).Error; err != nil {
				return images, err
			}
		}
	
		if builder.Parent != nil {
			// Save relations from image to parent
			relation := model.Relation{
				Parent:         *builder.Parent,
				Child:          image.ID,
				ParentCategory: builder.Category,
				ChildCategory:  "images",
			}
			if err := tx.Create(&relation).Error; err != nil {
				return images, err
			}
		}

		// Save tags (relations to tag table entries)
		for _, tagValue := range builder.Tags {
			tag := model.Tag{
				ID:     uuid.New(),
				Value:  tagValue,
				Entity: image.ID,
			}
			if err := tx.Create(&tag).Error; err != nil {
				return images, err
			}
		}

		images = append(images, image)
	}

	return images, nil
}

func CreateImage(c *gin.Context) {
	var builder ImageBuilder
	user, uerr := auth.GetUserFromClaims(db.DB, c);
	if uerr != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "unauthorized"})
		return
	}
	if err := c.ShouldBindJSON(&builder); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid JSON: " + err.Error()})
		return
	}

	tx := db.DB.Begin()
	if tx.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to create transaction"})
		return
	}
	images, err := CreateNewImage(tx, builder, user.ID, user.DefaultGroup)
	if err != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create image: " + err.Error()})
		return
	}
	tx.Commit()
	c.JSON(http.StatusOK, images)
}

func GetImagesByParentID(db *gorm.DB, parent uuid.UUID) ([]model.Image, error) {
	var images []model.Image

	err := db.
		Table("images").
		Joins("JOIN relations ON relations.child = images.id").
		Where("relations.parent = ? AND relations.child_category = ?", parent, "images").
		Find(&images).Error

	return images, err
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

func GetDefaultImage(parentCategory string, entity uuid.UUID) (model.Image, error) {
	var image model.Image
	err := db.DB.
		Table("images").
		Joins("JOIN relations ON relations.child = images.uuid").
		Where("relations.parent = ? AND relations.parent_category = ? AND relations.child_category = ? AND relations.comment = ?", entity, parentCategory, "images", "thumbnail").
		First(&image).Error

	if err != nil {
		return model.Image{}, err
	}

	return image, nil
}

func GetImage(c *gin.Context) {
	
	id := c.Param("id")

	var image model.Image
	if err := db.DB.First(&image, "id = ?", id).Error; err != nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "image not found"})
		return
	}

	var rawExifTags []model.ExifTag

	if err := db.DB.
		Table("exif_tags").
		Select("tag, value").
		Where("image = ?", image.ID).
		Scan(&rawExifTags).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to load EXIF tags"})
		return
	}

	var tags []string
	if err := db.DB.
		Table("tags").
		Select("value").
		Where("entity = ?", image.ID).
		Pluck("value", &tags).Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "failed to load tags"})
		return
	}

	exifTags := make([]ExifTagRender, 0, len(rawExifTags))
	for _, tag := range rawExifTags {
		exifTags = append(exifTags, ExifTagRender{
			Tag:   ExifTagName(int(tag.Tag)), // your Go-side tag name conversion
			Value: tag.Value,
		})
	}

	// Build render response
	render := ImageRender{
		ID:         image.ID,
		Url:		image.URL,
		Description: image.Description,
		Tags:        tags,
		ExifTags:   exifTags,
	}

	c.JSON(http.StatusOK, render)
}

func ListImages(c *gin.Context) {
	user, err := auth.GetUserFromClaims(db.DB, c)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": err.Error()})
		return
	}

	var images []model.Image

	imgerr := db.DB.
		Table("images").
		Select("images.id, images.url, images.description").
		Joins("JOIN relations ON relations.child = images.id AND relations.child_category = ?", "images").
		Joins("JOIN entities ON relations.parent = entities.id").
		Joins("JOIN group_rel ON group_rel.group_id = entities.group_id").
		Joins("JOIN users ON users.id = group_rel.user_id").
		Where("users.id = ?", user.ID).
		Scan(&images).Error

	if imgerr != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": imgerr})
		return
	}
	c.JSON(http.StatusOK, images)
}