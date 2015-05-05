// Copyright 2015 The tiny-http Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::Result as IoResult;
use std::io::{Read, Write};

use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

pub struct SequentialReaderBuilder<R> where R: Read + Send {
    reader: Arc<Mutex<R>>,
    next_trigger: Option<Receiver<()>>,
}

pub struct SequentialReader<R> where R: Read + Send {
    trigger: Option<Receiver<()>>,
    reader: Arc<Mutex<R>>,
    on_finish: Sender<()>,
}

pub struct SequentialWriterBuilder<W> where W: Write + Send {
    writer: Arc<Mutex<W>>,
    next_trigger: Option<Receiver<()>>,
}

pub struct SequentialWriter<W> where W: Write + Send {
    trigger: Option<Receiver<()>>,
    writer: Arc<Mutex<W>>,
    on_finish: Sender<()>,
}

impl<R: Read + Send> SequentialReaderBuilder<R> {
    pub fn new(reader: R) -> SequentialReaderBuilder<R> {
        SequentialReaderBuilder {
            reader: Arc::new(Mutex::new(reader)),
            next_trigger: None,
        }
    }
}

impl<W: Write + Send> SequentialWriterBuilder<W> {
    pub fn new(writer: W) -> SequentialWriterBuilder<W> {
        SequentialWriterBuilder {
            writer: Arc::new(Mutex::new(writer)),
            next_trigger: None,
        }
    }
}

impl<R: Read + Send> Iterator for SequentialReaderBuilder<R> {
    type Item = SequentialReader<R>;
    fn next(&mut self) -> Option<SequentialReader<R>> {
        let (tx, rx) = channel();
        let mut next_next_trigger = Some(rx);
        ::std::mem::swap(&mut next_next_trigger, &mut self.next_trigger);

        Some(SequentialReader {
            trigger: next_next_trigger,
            reader: self.reader.clone(),
            on_finish: tx,
        })
    }
}

impl<W: Write + Send> Iterator for SequentialWriterBuilder<W> {
    type Item = SequentialWriter<W>;
    fn next(&mut self) -> Option<SequentialWriter<W>> {
        let (tx, rx) = channel();
        let mut next_next_trigger = Some(rx);
        ::std::mem::swap(&mut next_next_trigger, &mut self.next_trigger);

        Some(SequentialWriter {
            trigger: next_next_trigger,
            writer: self.writer.clone(),
            on_finish: tx,
        })
    }
}

impl<R: Read + Send> Read for SequentialReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.trigger.as_mut().map(|v| v.recv().unwrap());
        self.trigger = None;

        self.reader.lock().unwrap().read(buf)
    }
}

impl<W: Write + Send> Write for SequentialWriter<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.trigger.as_mut().map(|v| v.recv().unwrap());
        self.trigger = None;

        self.writer.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.trigger.as_mut().map(|v| v.recv().unwrap());
        self.trigger = None;

        self.writer.lock().unwrap().flush()
    }
}

impl<R> Drop for SequentialReader<R> where R: Read + Send {
    fn drop(&mut self) {
        self.on_finish.send(()).ok();
    }
}

impl<W> Drop for SequentialWriter<W> where W: Write + Send {
    fn drop(&mut self) {
        self.on_finish.send(()).ok();
    }
}
