package main

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strings"
)

type CfError struct {
	Code       int `json: "code"`
	ErrorChain []struct {
		Code    int64  `json:"code"`
		Message string `json:"message"`
	} `json:"error_chain"`
	Message string `json: "message"`
}

type CfGenericResponse struct {
	Success bool      `json: "success"`
	Errors  []CfError `json: "errors"`
}

type ZonesResponse struct {
	Result []struct {
		Id     string `json: "id"`
		Name   string `json: "name"`
		Status string `json: "status"`
	} `json: "result"`
	Success bool      `json: "success"`
	Errors  []CfError `json: "errors"`
}

type DnsRecord struct {
	Id      string `json: "id"`
	Name    string `json: "name"`
	Type    string `json: "type"`
	Content string `json: "content"`
}

type DnsResponse struct {
	Result  []DnsRecord `json: "result"`
	Success bool        `json: "success"`
	Errors  []CfError   `json: "errors"`
}

type IpResponse struct {
	As          string  `json:"as"`
	City        string  `json:"city"`
	Country     string  `json:"country"`
	CountryCode string  `json:"countryCode"`
	Isp         string  `json:"isp"`
	Lat         float64 `json:"lat"`
	Lon         float64 `json:"lon"`
	Org         string  `json:"org"`
	Query       string  `json:"query"`
	Region      string  `json:"region"`
	RegionName  string  `json:"regionName"`
	Status      string  `json:"status"`
	Timezone    string  `json:"timezone"`
	Zip         string  `json:"zip"`
}

type DnsUpdateRequest struct {
	Type    string `json: "type"`
	Name    string `json: "name"`
	Content string `json: "content"`
	Ttl     int    `json: "ttl"`
}

const (
	EMAIL    = "@email"
	TOKEN    = "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
	ZONE     = "laspruca.nz"
	DNS      = "devtron.laspruca.nz"
	API_BASE = "https://api.cloudflare.com/client/v4"
)

func AddHeaders(req *http.Request) {
	req.Header.Add("X-Auth-Email", EMAIL)
	req.Header.Add("Authorization", fmt.Sprintf("Bearer %s", TOKEN))
	req.Header.Add("Content-Type", "application/json")
}

func RunRequest(req *http.Request) ([]byte, error) {
	client := http.Client{}
	res, err := client.Do(req)

	if err != nil {
		return []byte{}, err
	}

	defer res.Body.Close()
	return io.ReadAll(res.Body)
}

func Put(url string, body string) ([]byte, error) {
	req, err := http.NewRequest("GET", url, strings.NewReader(body))

	if err != nil {
		return []byte{}, err
	}

	AddHeaders(req)

	return RunRequest(req)
}

func Get(url string, headers bool) ([]byte, error) {
	req, err := http.NewRequest("GET", url, nil)

	if err != nil {
		return []byte{}, err
	}

	if headers {
		AddHeaders(req)
	}

	return RunRequest(req)
}

func main() {
	zonesBytes, err := Get(fmt.Sprintf("%s/zones", API_BASE), true)

	if err != nil {
		log.Fatalln(err)
	}

	var zones ZonesResponse

	json.Unmarshal(zonesBytes, &zones)

	if zones.Success == false {
		log.Fatalln(zones.Errors)
	}

	zoneId := ""

	for _, zone := range zones.Result {
		if zone.Name == ZONE {
			zoneId = zone.Id
		}
	}

	if zoneId == "" {
		log.Fatalln(fmt.Sprintf("Cloud not find zone %s", ZONE))
	}

	dnsBody, err := Get(fmt.Sprintf("%s/zones/%s/dns_records/", API_BASE, zoneId), true)

	if err != nil {
		log.Fatalln(err)
	}

	var dns_records DnsResponse
	json.Unmarshal(dnsBody, &dns_records)

	dnsId := DnsRecord{
		Name: "",
	}

	for _, dns_record := range dns_records.Result {
		if dns_record.Name == DNS && dns_record.Type == "A" {
			dnsId = dns_record
		}
	}

	if dnsId.Name == "" {
		log.Fatalln(fmt.Sprintf("Cloud not find dns record %s", DNS))
	}

	ipAddrBody, err := Get("http://ip-api.com/json/", false)

	var ipAddr IpResponse
	json.Unmarshal(ipAddrBody, &ipAddr)

	if ipAddr.Status != "success" {
		log.Fatalln("Could not get ip address")
	}

	if dnsId.Content != ipAddr.Query {
		req, err := json.Marshal(DnsUpdateRequest{
			Name:    dnsId.Name,
			Type:    "A",
			Content: ipAddr.Query,
			Ttl:     1,
		})

		if err != nil {
			log.Fatalln(err)
		}

		res, err := Put(fmt.Sprintf("%s/zones/%s/dns_records/%s", API_BASE, zoneId, dnsId.Id), string(req))

		if err != nil {
			log.Fatalln(err)
		}

		var updateResponse CfGenericResponse
		json.Unmarshal(res, &updateResponse)

		if updateResponse.Success == false {
			log.Fatalln(updateResponse.Errors)
		}

		log.Printf("Updated DNS record to %s \n", ipAddr.Query)
	}
}
