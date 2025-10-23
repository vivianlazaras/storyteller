gen_model() {
	gentool 
	  -dsn='host=localhost user=storyteller password=password dbname=storyteller port=5432 sslmode=disable' \ 
	-db='postgres' \
	-outPath='./models' \
	-onlyModel \
	-fieldNullable \
	-fieldWithTypeTag
}

echo "this script currently doesn't work bc I can't specify a custom type map to the CMD tool"
