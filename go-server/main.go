package main

import (
	"crypto/tls"
	"crypto/x509"
	"flag"
	"fmt"
	"gopkg.in/yaml.v2"
	"io/ioutil"
	"log"
	"net/http"
	"os"
)

type ServerConf struct {
	Addr                  string `yaml:"addr"`
	ServerCertificatePath string `yaml:"cert"`
	ServerKeyPath  string `yaml:"key"`
	ClientCertPath string `yaml:"client_cert"`
}

func LoadServerConf(path string) (*ServerConf, error) {
	in, err := ioutil.ReadFile(path)
	if err != nil {
		return nil, err
	}
	cfg := &ServerConf{}
	if err := yaml.Unmarshal(in, cfg); err != nil {
		return nil, err
	}
	return cfg, nil
}

func CertPoolFromFile(path string) (cert *x509.CertPool) {
	clientCert, err := os.ReadFile(path)
	if err != nil {
		log.Fatal(err)
	}
	clientCertPool := x509.NewCertPool()
	if ok := clientCertPool.AppendCertsFromPEM(clientCert); !ok {
		log.Fatalf("unable to parse cert from %s", path)
	}
	return clientCertPool
}

func TlsVersionFromString(version string) uint16 {
	if version == "1.2" {
		return tls.VersionTLS12
	}
	return tls.VersionTLS13
}

func main() {
	confFile := flag.String("conf", "conf.yml", "config file")
	tlsVersion := flag.String("tls-version", "1.3", "max tls version, default \"1.3\"")
	flag.Parse()
	conf, err := LoadServerConf(*confFile)
	if err != nil {
		log.Fatal(err)
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/", func(w http.ResponseWriter, req *http.Request) {
		if req.URL.Path != "/" {
			http.NotFound(w, req)
			return
		}
		fmt.Fprintf(w, "Serving HTTPS with mTLS\n")
	})

	srv := &http.Server{
		Addr:    conf.Addr,
		Handler: mux,
		TLSConfig: &tls.Config{
			MaxVersion: TlsVersionFromString(*tlsVersion),
			ClientCAs:  CertPoolFromFile(conf.ClientCertPath), // trusted client cert
			ClientAuth: tls.RequireAndVerifyClientCert,
		},
	}
	fmt.Printf("Starting server on %s\n", conf.Addr)

	err = srv.ListenAndServeTLS(conf.ServerCertificatePath, conf.ServerKeyPath)
	log.Fatal(err)
}
