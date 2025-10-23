package config

import (
	"encoding/json"
	"fmt"
	"os"
)

type DBConfig struct {
	Backend string		`json:"backend"`
	Name string			`json:"name"`
	User string			`json:"user"`
	Host string			`json:"host"`
	Port uint16			`json:"port"`
	PasswordFile string	`json:"passwordFile"`
}

type OIDCConfig struct {
	ClientID string		`json:"client_id"`
	SecretFile string	`json:"client_secret"`
	IssuerURL string	`json:"issuer_url"`
	Redirect string		`json:"redirect"`
}

type ServerConfig struct {
	Listen string		`json:"listen"`
	Port uint16			`json:"port"`
	Ssl bool			`json:"ssl"`
	Url string			`json:"url"`
	CertFile *string	`json:"certFile"`
	KeyFile	 *string	`json:"keyFile"`
	// if the go API should be handling issuing JWTs
	SelfHostedAuth bool		`json:"self_hosted_auth"`
	Oidc	 *[]OIDCConfig `json:"oidc"`
}

type APISettings struct {
	AutoAddUsers bool	`json:"auto_craete_users"`
}

type APIConfig struct {
	Server ServerConfig	`json:"server"`
	DB DBConfig		`json:"db"`
}

type Config struct {
	Server ServerConfig	`json:"server"`
	Api APIConfig		`json:"api"`
}

func LoadConfig(path string) (Config, error) {
	var cfg Config

	file, err := os.Open(path)
	if err != nil {
		return cfg, fmt.Errorf("failed to open config file: %w", err)
	}
	defer file.Close()

	decoder := json.NewDecoder(file)
	if err := decoder.Decode(&cfg); err != nil {
		return cfg, fmt.Errorf("failed to decode config: %w", err)
	}

	return cfg, nil
}