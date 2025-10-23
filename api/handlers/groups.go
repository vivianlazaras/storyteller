package handlers

import (
	"github.com/google/uuid"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/model"
	"github.com/vivianlazaras/storyteller/auth"
	"github.com/gin-gonic/gin"
	"net/http"
    "gorm.io/gorm"
	"gorm.io/gorm/clause"
)

type GroupBuilder struct {
	Name     string    `json:"name" binding:"required"`
	Hidden   bool      `json:"hidden"`
	Propagate bool		`json:"propagate"`
	ParentID *uuid.UUID `json:"parent_id"` // nullable
	UserIDs  []uuid.UUID `json:"user_ids"`
	Permissions []string `json:"permissions"`
	Description *string	`json:"description"`
}

func RegisterGroupRoutes(r *gin.Engine) *gin.Engine {
	r.GET("/groups/", auth.JWTMiddleware(), ListGroups);
	return r
}

// checks if the user is in the group
func HasAccess(db *gorm.DB, userID uuid.UUID, groupID *uuid.UUID) (bool, error) {
	
	if groupID == nil {
		return false, nil
	}
	
	var count int64
	err := db.
		Table("group_rel").
		Joins("JOIN group_closure ON group_rel.group_id = group_closure.ancestor_id").
		Where("group_rel.user_id = ? AND group_closure.descendant_id = ?", userID, *groupID).
		Count(&count).Error

	return count > 0, err
}

func CreateDefaultGroup(db *gorm.DB, userID uuid.UUID, name string) (*model.Group, error) {
	
	var request = GroupBuilder {
		Name: name,
		Hidden: false,
		Propagate: true,
		ParentID: nil,
		Description: nil,
		UserIDs: []uuid.UUID{userID},
		Permissions: []string{"create", "read", "update", "delete"},
	}

	return CreateNewGroup(db, request)
}

func GetGroupsForUser(db *gorm.DB, userID uuid.UUID) ([]model.Group, error) {
	var groups []model.Group

	err := db.Table("groups").
		Joins("JOIN group_rel ON groups.id = group_rel.group_id").
		Where("group_rel.user_id = ?", userID).
		Scan(&groups).Error

	if err != nil {
		return nil, err
	}
	return groups, nil
}

func CreateNewGroup(db *gorm.DB, req GroupBuilder) (*model.Group, error) {
	group := &model.Group{ID: uuid.New(), Name: &req.Name, Hidden: req.Hidden}
	if err := db.Create(group).Error; err != nil {
		return nil, err
	}

	// Add self to group_closure (depth=0)
	closure := model.GroupClosure{AncestorID: group.ID, DescendantID: group.ID, Depth: 0}
	if err := db.Create(&closure).Error; err != nil {
		return nil, err
	}

	// Handle parent group linkage (group_closure)
	if req.ParentID != nil {
		adderr := AddSubgroup(db, group.ID, *req.ParentID)
		if adderr != nil {
			return nil, adderr
		}
	}

	// Add users to model.GroupRel
	for _, uid := range req.UserIDs {
		err := AddUserToGroup(db, uid, group.ID);
		if err != nil {
			return nil, err
		}
	}

	for _, perm := range req.Permissions {
		entry := model.GroupPermission{
			GroupID:   group.ID,
			Permission: perm,
			Propagate:  req.Propagate,
		}
		if err := db.Create(&entry).Error; err != nil {
			return nil, err
		}
	}

	return group, nil
}

func AddUserToGroup(db *gorm.DB, userID, groupID uuid.UUID) error {
	if err := db.Create(&model.GroupRel{
		UserID: userID,
		GroupID: groupID,
	}).Error; err != nil {
		return err
	}

	return nil
}

func AddSubgroup(db *gorm.DB, parentID, childID uuid.UUID) error {
	return db.Transaction(func(tx *gorm.DB) error {
		// insert all (ancestor of parent, descendant of child) pairs
		var superAncestors []model.GroupClosure
		var subDescendants []model.GroupClosure

		if err := tx.Where("descendant_id = ?", parentID).Find(&superAncestors).Error; err != nil {
			return err
		}
		if err := tx.Where("ancestor_id = ?", childID).Find(&subDescendants).Error; err != nil {
			return err
		}

		for _, a := range superAncestors {
			for _, d := range subDescendants {
				entry := model.GroupClosure{
					AncestorID:   a.AncestorID,
					DescendantID: d.DescendantID,
					Depth:        a.Depth + 1 + d.Depth,
				}
				_ = tx.Clauses(clause.OnConflict{DoNothing: true}).Create(&entry)
			}
		}

		return nil
	})
}

func ShareEntityBetweenGroups(db *gorm.DB, entityID, groupAID, groupBID uuid.UUID, userIDs []uuid.UUID) error {
	return db.Transaction(func(tx *gorm.DB) error {
		var sharedGroup model.Group

		// Try to find existing shared model.group (non-hidden, same child of both groups)
		err := tx.Raw(`
			SELECT g.* FROM groups g
			JOIN group_closure gc1 ON gc1.descendant_id = g.id AND gc1.ancestor_id = ?
			JOIN group_closure gc2 ON gc2.descendant_id = g.id AND gc2.ancestor_id = ?
			WHERE g.hidden = true
			LIMIT 1
		`, groupAID, groupBID).Scan(&sharedGroup).Error

		if err != nil {
			return err
		}

		// If not found, create a new hidden group
		if sharedGroup.ID == uuid.Nil {
			var groupName = "shared_" + uuid.New().String();
			sharedGroup = model.Group{Name: &groupName, Hidden: true}
			if err := tx.Create(&sharedGroup).Error; err != nil {
				return err
			}
			if err := tx.Create(&model.GroupClosure{AncestorID: sharedGroup.ID, DescendantID: sharedGroup.ID, Depth: 0}).Error; err != nil {
				return err
			}
			if err := AddSubgroup(tx, groupAID, sharedGroup.ID); err != nil {
				return err
			}
			if err := AddSubgroup(tx, groupBID, sharedGroup.ID); err != nil {
				return err
			}
		}

		// Attach entity to shared group
		if err := tx.Clauses(clause.OnConflict{DoNothing: true}).Create(&model.EntityGroup{
			EntityID: entityID,
			GroupID:  sharedGroup.ID,
		}).Error; err != nil {
			return err
		}

		// Add users to shared model.GroupRel (optional for permission clarity/perf)
		for _, uid := range userIDs {
			gr := model.GroupRel{UserID: uid, GroupID: sharedGroup.ID}
			_ = tx.Clauses(clause.OnConflict{DoNothing: true}).Create(&gr)
		}

		return nil
	})
}

func CheckUserGroupPermission(db *gorm.DB, userID, groupID uuid.UUID, permissionName string) (bool, error) {
	// 1. Check access via primary group
    hasAccess, err := HasAccess(db, userID, &groupID)
    if err != nil {
        return false, err
    }
    if hasAccess {
        var count int64
        err := db.
            Table("group_permissions").
            Where("group_id = ? AND permission = ?", groupID, permissionName).
            Count(&count).Error
        if err != nil {
            return false, err
        }
        if count > 0 {
			return true, nil
		}
    }

	return false, nil
}

// CheckUserEntityPermission returns whether user has a specific permission on the entity
// and which groups grant that access (primary or shared).
func CheckUserEntityPermission(
    db *gorm.DB,
    userID, entityID uuid.UUID,
    permissionName string,
) (bool, []uuid.UUID, error) {
    var entity model.Entity
    if err := db.First(&entity, "id = ?", entityID).Error; err != nil {
        return false, nil, err
    }

    var authorizedGroups []uuid.UUID

    // 1. Check access via primary group
    hasAccess, err := HasAccess(db, userID, entity.GroupID)
    if err != nil {
        return false, nil, err
    }
    if hasAccess {
        var count int64
        err := db.
            Table("group_permissions").
            Where("group_id = ? AND permission = ?", entity.GroupID, permissionName).
            Count(&count).Error
        if err != nil {
            return false, nil, err
        }
        if count > 0 {
			// the dereference is safe here because the nullability of entity.GroupID was checked in HasAccess
            authorizedGroups = append(authorizedGroups, *entity.GroupID)
			return true, authorizedGroups, nil
		}
    }

    // 2. Check access via shared groups
    var sharedGroupIDs []uuid.UUID
    err = db.
        Table("entity_groups as eg").
        Select("eg.group_id").
        Joins("JOIN group_rel gr ON gr.group_id = eg.group_id").
        Joins("JOIN group_closure gc ON gc.ancestor_id = gr.group_id AND gc.descendant_id = eg.group_id").
        Where("eg.entity_id = ? AND gr.user_id = ?", entityID, userID).
        Distinct().
        Scan(&sharedGroupIDs).Error
    if err != nil {
        return false, nil, err
    }

    // Check if any shared group grants the requested permission
    if len(sharedGroupIDs) > 0 {
        var permittedSharedGroupIDs []uuid.UUID
        err := db.
            Table("group_permissions").
            Select("group_id").
            Where("group_id IN ? AND permission = ?", sharedGroupIDs, permissionName).
            Scan(&permittedSharedGroupIDs).Error
        if err != nil {
            return false, nil, err
        }

        authorizedGroups = append(authorizedGroups, permittedSharedGroupIDs...)
    }

    if len(authorizedGroups) > 0 {
        return true, authorizedGroups, nil
    }

    return false, nil, nil
}

func CreateGroup(c *gin.Context) {
	var req GroupBuilder
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "Invalid request: " + err.Error()})
		return
	}

	tx := db.DB.Begin()
	if tx.Error != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to start database transaction"})
		return
	}

	group, err := CreateNewGroup(tx, req); 
	if err != nil {
		tx.Rollback()
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to create group: " + err.Error()})
		return
	}

	if err := tx.Commit().Error; err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "Failed to commit transaction: " + err.Error()})
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"group_id": group.ID,
	})
}

func ListGroups(c *gin.Context) {
	user, err := auth.GetUserFromClaims(db.DB, c)
	if err != nil {
		c.JSON(http.StatusUnauthorized, err)
		return
	}

	groups, uerr := GetGroupsForUser(db.DB, user.ID)

	if uerr != nil {
		c.JSON(http.StatusInternalServerError, uerr)
		return
	}
	
	c.JSON(http.StatusOK, groups)
}