package flatgeobuf

import (
	"syscall"
	"unsafe"
)

// madvise calls madvise directly on MacOS as it is not available in Go's
// syscall package.
func madvise(b []byte, advice int) error {
	// MacOS handles page alignment by itself, so we don't need to do it here
	// and can call madvise with any arbitrary slice.
	_, _, err := syscall.Syscall(syscall.SYS_MADVISE, uintptr(unsafe.Pointer(&b[0])),
		uintptr(len(b)), uintptr(advice))
	if err != 0 {
		return err
	}

	return nil
}
