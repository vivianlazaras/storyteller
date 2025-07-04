// Code generated by gorm.io/gen. DO NOT EDIT.
// Code generated by gorm.io/gen. DO NOT EDIT.
// Code generated by gorm.io/gen. DO NOT EDIT.

package model

const TableNameRecording = "recordings"

// Recording mapped from table <recordings>
type Recording struct {
	ID          string `gorm:"column:id;primaryKey;default:gen_random_uuid()" json:"id"`
	Title       string `gorm:"column:title" json:"title"`
	Artist      string `gorm:"column:artist" json:"artist"`
	Album       string `gorm:"column:album" json:"album"`
	Genre       string `gorm:"column:genre" json:"genre"`
	Year        int32  `gorm:"column:year" json:"year"`
	TrackNumber int32  `gorm:"column:track_number" json:"track_number"`
	DiscNumber  int32  `gorm:"column:disc_number" json:"disc_number"`
	AlbumArtist string `gorm:"column:album_artist" json:"album_artist"`
	Comment     string `gorm:"column:comment" json:"comment"`
	Composer    string `gorm:"column:composer" json:"composer"`
	Lyrics      string `gorm:"column:lyrics" json:"lyrics"`
	Created     int64  `gorm:"column:created;default:unix_now()" json:"created"`
	LastEditted int64  `gorm:"column:last_editted;default:unix_now()" json:"last_editted"`
}

// TableName Recording's table name
func (*Recording) TableName() string {
	return TableNameRecording
}
