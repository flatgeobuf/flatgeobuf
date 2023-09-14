package writer

// HeaderUpdater is an interface for updating the Header after all entries for
// the flatgeobuf file have been processed. Usefull to set metadata that depends
// on what has been added.
type HeaderUpdater interface {
	Update(header *Header)
}
