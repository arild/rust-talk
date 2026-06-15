package main

// DTOs mirror the Rust controller::response types one-for-one: field order
// determines the wire-format key order, and `omitempty` on pointer fields
// produces the same "skip when None" behaviour Rust gets from
// `#[serde(skip_serializing_if = "Option::is_none")]`.

type ParcelResponse struct {
	ParcelNumber                   string                          `json:"parcelNumber"`
	ConsignmentNumber              string                          `json:"consignmentNumber"`
	Status                         string                          `json:"status"`
	UserChosenName                 *string                         `json:"userChosenName,omitempty"`
	Direction                      string                          `json:"direction"`
	Transport                      *TransportResponse              `json:"transport,omitempty"`
	Dimensions                     *DimensionsResponse             `json:"dimensions,omitempty"`
	WeightInKg                     *Float                          `json:"weightInKg,omitempty"`
	Delivery                       *DeliveryResponse               `json:"delivery,omitempty"`
	Sender                         *SenderResponse                 `json:"sender,omitempty"`
	Recipient                      *RecipientResponse              `json:"recipient,omitempty"`
	ProductName                    string                          `json:"productName"`
	ProductGroup                   string                          `json:"productGroup"`
	Events                         []EventResponse                 `json:"events"`
	Features                       []FeatureResponse               `json:"features"`
	ExpiresAt                      Instant                         `json:"expiresAt"`
	ParcelNumbersInConsignment     []string                        `json:"parcelNumbersInConsignment"`
	CustomsTax                     *CustomsTaxResponse             `json:"customsTax,omitempty"`
	CustomsInformationRequirements *CustomsInformationRequirements `json:"customsInformationRequirements,omitempty"`
	Rewards                        *RewardsResponse                `json:"rewards,omitempty"`
}

type TransportResponse struct {
	Type     string `json:"type"`
	Electric bool   `json:"electric"`
	FuelType string `json:"fuelType"`
}

type DimensionsResponse struct {
	LengthInCm int `json:"lengthInCm"`
	WidthInCm  int `json:"widthInCm"`
	HeightInCm int `json:"heightInCm"`
}

type DeliveryResponse struct {
	Type                            string                `json:"type"`
	PickUpPointID                   *string               `json:"pickUpPointId,omitempty"`
	DeadlineDate                    *Instant              `json:"deadlineDate,omitempty"`
	ExtendDeadlineTo                *Instant              `json:"extendDeadlineTo,omitempty"`
	PickUpCode                      *string               `json:"pickUpCode,omitempty"`
	PickUpQrCode                    *string               `json:"pickUpQrCode,omitempty"`
	ShelfNumber                     *string               `json:"shelfNumber,omitempty"`
	GateCode                        *string               `json:"gateCode,omitempty"`
	PinCode                         *string               `json:"pinCode,omitempty"`
	QrCode                          *string               `json:"qrCode,omitempty"`
	Permission                      string                `json:"permission"`
	BankidAuthenticated             string                `json:"bankidAuthenticated"`
	Options                         []string              `json:"options"`
	ProgressPercentageBasedOnEvents int                   `json:"progressPercentageBasedOnEvents"`
	DeliveryTime                    *DeliveryTimeResponse `json:"deliveryTime,omitempty"`
}

type DeliveryTimeResponse struct {
	Date           Instant                 `json:"date"`
	DeliveryWindow *DeliveryWindowResponse `json:"deliveryWindow,omitempty"`
}

type DeliveryWindowResponse struct {
	Start Instant `json:"start"`
	End   Instant `json:"end"`
}

type SenderResponse struct {
	Name            string                    `json:"name"`
	IconURL         *string                   `json:"iconUrl,omitempty"`
	PhoneNumber     *string                   `json:"phoneNumber,omitempty"`
	Email           *string                   `json:"email,omitempty"`
	CustomerNumber  *string                   `json:"customerNumber,omitempty"`
	Branding        *BrandingResponse         `json:"branding,omitempty"`
	CustomerService *CustomerServiceResponse  `json:"customerService,omitempty"`
}

type BrandingResponse struct {
	BannerImage  *ImageResponse `json:"bannerImage,omitempty"`
	ContentImage *ImageResponse `json:"contentImage,omitempty"`
	Title        *string        `json:"title,omitempty"`
	Description  *string        `json:"description,omitempty"`
	StoreLink    *LinkResponse  `json:"storeLink,omitempty"`
}

type ImageResponse struct {
	URL  string  `json:"url"`
	Text *string `json:"text,omitempty"`
}

type LinkResponse struct {
	URL  string  `json:"url"`
	Text *string `json:"text,omitempty"`
}

type CustomerServiceResponse struct {
	Name string `json:"name"`
	URL  string `json:"url"`
}

type RecipientResponse struct {
	Name                     *string `json:"name,omitempty"`
	PostalCode               *string `json:"postalCode,omitempty"`
	City                     *string `json:"city,omitempty"`
	Address                  *string `json:"address,omitempty"`
	PhoneNumber              *string `json:"phoneNumber,omitempty"`
	Email                    *string `json:"email,omitempty"`
	SharedAccessPhoneNumber  *string `json:"sharedAccessPhoneNumber,omitempty"`
	BankidAuthenticated      bool    `json:"bankidAuthenticated"`
}

type EventResponse struct {
	Description          string  `json:"description"`
	Date                 Instant `json:"date"`
	City                 *string `json:"city,omitempty"`
	CountryCode          *string `json:"countryCode,omitempty"`
	Type                 string  `json:"type"`
	Cause                *string `json:"cause,omitempty"`
	WhatsNextDescription *string `json:"whatsNextDescription,omitempty"`
	DisplayStatus        string  `json:"displayStatus"`
}

type FeatureResponse struct {
	Type        string   `json:"type"`
	URL         *string  `json:"url,omitempty"`
	Title       *string  `json:"title,omitempty"`
	Description *string  `json:"description,omitempty"`
	Date        *Instant `json:"date,omitempty"`
}

type CustomsTaxResponse struct {
	Type              string              `json:"type"`
	Status            string              `json:"status"`
	TotalAmountInOre  int                 `json:"totalAmountInOre"`
	ParcelContent     []ParcelContentItem `json:"parcelContent"`
	CustomsPriceList  []CustomsPriceItem  `json:"customsPriceList"`
	DueDate           *Instant            `json:"dueDate,omitempty"`
}

type ParcelContentItem struct {
	Description     string `json:"description"`
	CurrencyCode    string `json:"currencyCode"`
	AmountInSubunit int    `json:"amountInSubunit"`
}

type CustomsPriceItem struct {
	Description  string `json:"description"`
	AmountInOre  int    `json:"amountInOre"`
}

type CustomsInformationRequirements struct {
	DocumentsRequired      bool `json:"documentsRequired"`
	InformationProvided    bool `json:"informationProvided"`
	IdentificationRequired bool `json:"identificationRequired"`
}

type RewardsResponse struct {
	RewardsEarnings []RewardsEarningResponse `json:"rewardsEarnings"`
}

type RewardsEarningResponse struct {
	ValidFrom Instant `json:"validFrom"`
	ValidTo   Instant `json:"validTo"`
	Type      string  `json:"type"`
	Text      string  `json:"text"`
	Coins     int     `json:"coins"`
}
