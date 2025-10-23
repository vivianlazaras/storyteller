package handlers

import (
    "net/http"
	"fmt"
    "github.com/gin-gonic/gin"
    "github.com/google/uuid"
    "github.com/vivianlazaras/storyteller/db"
    "github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/auth"
	"gorm.io/gorm"
)

func RegisterEntityRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/relations/:category", auth.JWTMiddleware(), ListEntitiesByCategory)
	r.POST("/relations/", auth.JWTMiddleware(), CreateRelation)
	return r
}

type RelatedEntity struct {
    ID   string `json:"id"`
    Name string `json:"name"`
    Description string  `json:"description"`
}

type GroupedEntity struct {
	ID			uuid.UUID		`json:"id"`
	Name		string			`json:"name"`
	Entities	[]RelatedEntity	`json:"entities"`
}

func GetTableForCategory(category string) (string, error) {
	var table string
	switch category {
	case "story", "stories":
		table = "stories"
	case "character", "characters":
		table = "characters"
	case "timeline", "timelines":
		table = "timelines"
	case "location", "locations":
		table = "locations"
	case "fragment", "fragments":
		table = "fragments"
	default:
		return "", fmt.Errorf("unsupported category: %s", category)
	}
	return table, nil
}

func ListEntitiesByCategoryForUser(db *gorm.DB, userID uuid.UUID, category string) (map[string]GroupedEntity, error) {
	
	// Group by group_id
	grouped := make(map[string]GroupedEntity)

	var entities []struct {
		ID			string
		Name		string
		Description	string
		GroupID		uuid.UUID
		GroupName	string
	};

	table, cerr := GetTableForCategory(category)
	if cerr != nil {
		return nil, cerr
	}

	err := db.
		Table("entities").
		Select(fmt.Sprintf(
			"entities.id, %s.name, %s.description, groups.id as group_id, groups.name as group_name",
			table, table,
		)).
		Joins("JOIN groups ON groups.id = entities.group_id").
		Joins("JOIN group_rel ON group_rel.group_id = groups.id").
		Joins(fmt.Sprintf("JOIN %s ON %s.id = entities.id", table, table)).
		Where("group_rel.user_id = ?", userID).
		Order(fmt.Sprintf("%s.name ASC", table)).
		Scan(&entities).Error

	if err != nil {
		return nil, err
	}

	for _, e := range entities {
		key := fmt.Sprintf("%s (%s)", e.GroupName, e.GroupID.String())
	
		related := RelatedEntity{
			ID:          e.ID,
			Name:        e.Name,
			Description: e.Description,
		}
	
		if _, ok := grouped[key]; !ok {
			grouped[key] = GroupedEntity{
				ID:       e.GroupID,
				Name:     e.GroupName,
				Entities: []RelatedEntity{related},
			}
		} else {
			group := grouped[key]
			group.Entities = append(group.Entities, related)
			grouped[key] = group
		}
	}

	return grouped, nil
}

func ListEntitiesByCategoryForGroup(db *gorm.DB, groupID uuid.UUID, category string) ([]RelatedEntity, error) {
	var entities []RelatedEntity
	table, cerr := GetTableForCategory(category)
	if cerr != nil {
		return []RelatedEntity{}, cerr
	}

	err := db.
		Table("entities").
		Select(fmt.Sprintf(
			"entities.id, %s.name, %s.description",
			table, table,
		)).
		Joins(fmt.Sprintf("JOIN %s ON %s.id = entities.id", table, table)).
		Where("entities.group_id = ?", groupID).
		Scan(&entities).Error

	if err != nil {
		return nil, err
	}

	return entities, nil
}

func ListEntitiesByCategory(c *gin.Context) {
	category := c.Param("category")
	user, uerr := auth.GetUserFromClaims(db.DB, c)
	if uerr != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "failed to get user"})
		return
	}

	if category == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "child_category query parameter is required"})
		return
	}

	var entities []RelatedEntity
	var query string

	switch category {
	case "characters", "stories", "fragments", "locations", "timelines":
		query = fmt.Sprintf(`
			SELECT e.id, e.name, e.description
			FROM %s e
			JOIN entities ent ON ent.id = e.id
			JOIN group_rel gr ON gr.group_id = ent.group_id
			AND gr.user_id = ?
			ORDER BY e.name ASC
		`, category)
	default:
		c.JSON(http.StatusBadRequest, gin.H{"error": "unknown child_category"})
		return
	}

	err := db.DB.Raw(query, user.ID).Scan(&entities).Error
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, entities)
}

/// utility function to set group_id for entity
func CreateNewEntity(db *gorm.DB, id, user, group uuid.UUID) error {
	// check to make sure the user is authorized to create an entity within the group
	access, err := CheckUserGroupPermission(db, user, group, "create"); 
	if err != nil {
		fmt.Println("access denied with error");
		return err
	}
	if access != true {

		fmt.Println("access denied without err");
		return fmt.Errorf("access denied")
	}
	// Perform the update on the entities table
	result := db.Model(&model.Entity{}).
		Where("id = ?", id).
		Update("group_id", group)

	// Check if the update succeeded
	if result.Error != nil {
		return result.Error
	}
	if result.RowsAffected == 0 {
		return gorm.ErrRecordNotFound
	}
	return nil
}

func CreateNewRelation(db *gorm.DB, relation *model.Relation) (*model.Relation, error) {
	// Check if the relation already exists
	var exists bool
	err := db.
		Table("relations").
		Select("count(*) > 0").
		Where("parent = ? AND child = ? AND parent_category = ? AND child_category = ?",
			relation.Parent, relation.Child, relation.ParentCategory, relation.ChildCategory).
		Find(&exists).Error

	if err != nil {
		return nil, fmt.Errorf("failed to check existing relation: %w", err)
	}

	if exists {
		return nil, fmt.Errorf("relation already exists")
	}

	// Insert new relation
	if err := db.Table("relations").Create(&relation).Error; err != nil {
		return nil, fmt.Errorf("failed to create relation: %w", err)
	}

	return relation, nil
}

func CreateRelation(c *gin.Context) {
	var relation model.Relation

	if err := c.ShouldBindJSON(&relation); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request: " + err.Error()})
		return
	}

    _, err := CreateNewRelation(db.DB, &relation)
    if err != nil {
		fmt.Println("failed to create new relation %s", err);
		c.JSON(http.StatusInternalServerError, gin.H{"error": err})
		return
	}

	c.JSON(http.StatusOK, relation)
}

type Operation struct {
    Operation   string
    // create, read, update, delete (because I can)
    mode        string
}

/// used to ensure that the user has proper permissions to perform a write
func CheckPermissions(user uuid.UUID, entity uuid.UUID, op Operation) bool {
    return false
}

func DeleteEntity() {}

/// need to create a setGroup function

/// used to archive stories so they don't have to be deleted
func Archive(c *gin.Context) {
	//user, uerr := auth.GetUserFromClaims(db.DB, c);
	
}