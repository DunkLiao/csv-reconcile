use encoding_rs::{Decoder, Encoding};
use std::io::{self, Read};

pub struct DecodeReader<R: Read> {
    inner: R,
    decoder: Decoder,
    in_buf: Vec<u8>,
    in_pos: usize,
    in_len: usize,
    out_buf: Vec<u8>,
    out_pos: usize,
    out_len: usize,
    eof_reached: bool,
}

impl<R: Read> DecodeReader<R> {
    pub fn new(mut inner: R, encoding: &'static Encoding, skip_bytes: usize) -> io::Result<Self> {
        if skip_bytes > 0 {
            let mut skip = vec![0u8; skip_bytes];
            inner.read_exact(&mut skip)?;
        }

        Ok(Self {
            inner,
            decoder: encoding.new_decoder(),
            in_buf: vec![0u8; 32768],
            in_pos: 0,
            in_len: 0,
            out_buf: vec![0u8; 65536],
            out_pos: 0,
            out_len: 0,
            eof_reached: false,
        })
    }
}

impl<R: Read> Read for DecodeReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            // If we have decoded bytes in out_buf, copy them to buf
            if self.out_pos < self.out_len {
                let to_copy = std::cmp::min(buf.len(), self.out_len - self.out_pos);
                buf[..to_copy].copy_from_slice(&self.out_buf[self.out_pos..self.out_pos + to_copy]);
                self.out_pos += to_copy;
                return Ok(to_copy);
            }

            // out_buf is empty, reset positions
            self.out_pos = 0;
            self.out_len = 0;

            if self.eof_reached {
                return Ok(0);
            }

            // If in_buf is consumed or empty, read more from inner
            if self.in_pos >= self.in_len {
                self.in_pos = 0;
                let n = self.inner.read(&mut self.in_buf)?;
                self.in_len = n;
                if n == 0 {
                    // EOF from inner reader, run decode with last = true
                    let (_res, _read, written, _had_errors) =
                        self.decoder.decode_to_utf8(&[], &mut self.out_buf, true);
                    self.out_len = written;
                    self.eof_reached = true;
                    if written == 0 {
                        return Ok(0);
                    }
                    continue;
                }
            }

            // Decode from in_buf into out_buf
            let (_res, read, written, _had_errors) = self.decoder.decode_to_utf8(
                &self.in_buf[self.in_pos..self.in_len],
                &mut self.out_buf,
                false,
            );
            self.in_pos += read;
            self.out_len = written;
        }
    }
}
