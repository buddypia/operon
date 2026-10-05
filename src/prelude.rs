//! Shared external imports for the split modules.

pub(crate) use anyhow::{anyhow, Context, Result};
pub(crate) use directories::ProjectDirs;
pub(crate) use eframe::egui::{
    self, text::LayoutJob, Color32, FontId, RichText, TextFormat, TextStyle,
};
pub(crate) use egui_phosphor::regular as icons;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    fs,
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
pub(crate) use uuid::Uuid;

pub(crate) use crate::i18n::{detected_language, set_active_language, tr, Language};
pub(crate) use crate::tf;
