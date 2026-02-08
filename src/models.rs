/// Data models for the appointment scheduling system.
///
/// This module defines the core data structures used throughout the system:
/// - Priority: Enum for appointment urgency levels
/// - Patient: Patient information
/// - TimeSlot: Available time windows in the calendar
/// - Appointment: Confirmed appointment details
/// - AppointmentRequest: Patient request for an appointment

use chrono::{DateTime, Duration, Local};
use std::cmp::Ordering;
use uuid::Uuid;

/// Priority levels for appointments.
///
/// Higher numeric values indicate higher priority.
/// Emergency cases are scheduled first, followed by urgent,
/// then routine appointments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    Routine = 1,
    Urgent = 2,
    Emergency = 3,
}

impl Priority {
    /// Convert a string to a Priority enum value.
    pub fn from_string(value: &str) -> Result<Self, String> {
        match value.to_lowercase().trim() {
            "routine" => Ok(Priority::Routine),
            "urgent" => Ok(Priority::Urgent),
            "emergency" => Ok(Priority::Emergency),
            _ => Err(format!(
                "Invalid priority: '{}'. Must be one of: routine, urgent, emergency",
                value
            )),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Priority::Routine => "ROUTINE",
            Priority::Urgent => "URGENT",
            Priority::Emergency => "EMERGENCY",
        }
    }
}

/// Represents a patient in the scheduling system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patient {
    pub patient_id: String,
    pub name: String,
    pub contact: String,
}

impl Patient {
    /// Create a new patient with validation.
    pub fn new(patient_id: String, name: String, contact: String) -> Result<Self, String> {
        if patient_id.is_empty() {
            return Err("Patient ID cannot be empty".to_string());
        }
        if name.is_empty() {
            return Err("Patient name cannot be empty".to_string());
        }
        if contact.is_empty() {
            return Err("Patient contact cannot be empty".to_string());
        }

        Ok(Patient {
            patient_id,
            name,
            contact,
        })
    }
}

/// Represents an available time slot in the doctor's calendar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSlot {
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
    pub is_available: bool,
    pub slot_id: String,
}

impl TimeSlot {
    /// Create a new time slot with validation.
    pub fn new(start_time: DateTime<Local>, end_time: DateTime<Local>) -> Result<Self, String> {
        if end_time <= start_time {
            return Err("End time must be after start time".to_string());
        }

        Ok(TimeSlot {
            start_time,
            end_time,
            is_available: true,
            slot_id: Uuid::new_v4().to_string(),
        })
    }

    /// Calculate the duration of the time slot.
    pub fn duration(&self) -> Duration {
        self.end_time - self.start_time
    }

    /// Get the duration in minutes.
    pub fn duration_minutes(&self) -> i64 {
        self.duration().num_minutes()
    }

    /// Check if this time slot overlaps with another.
    pub fn overlaps_with(&self, other: &TimeSlot) -> bool {
        self.start_time < other.end_time && self.end_time > other.start_time
    }

    /// Check if a datetime falls within this time slot.
    pub fn contains(&self, dt: &DateTime<Local>) -> bool {
        &self.start_time <= dt && dt < &self.end_time
    }
}

impl std::hash::Hash for TimeSlot {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.slot_id.hash(state);
    }
}

/// Represents a confirmed appointment.
#[derive(Debug, Clone)]
pub struct Appointment {
    pub appointment_id: String,
    pub patient: Patient,
    pub time_slot: TimeSlot,
    pub priority: Priority,
    pub reason: String,
    pub created_at: DateTime<Local>,
    pub confirmed: bool,
}

impl Appointment {
    /// Create a new appointment with validation.
    pub fn new(
        patient: Patient,
        time_slot: TimeSlot,
        priority: Priority,
        reason: String,
    ) -> Result<Self, String> {
        if reason.is_empty() {
            return Err("Appointment reason cannot be empty".to_string());
        }

        Ok(Appointment {
            appointment_id: Uuid::new_v4().to_string(),
            patient,
            time_slot,
            priority,
            reason,
            created_at: Local::now(),
            confirmed: true,
        })
    }
}

/// Represents a patient's request for an appointment.
#[derive(Debug, Clone)]
pub struct AppointmentRequest {
    pub request_id: String,
    pub patient: Patient,
    pub priority: Priority,
    pub preferred_time: DateTime<Local>,
    pub reason: String,
    pub flexibility_minutes: i64,
    pub created_at: DateTime<Local>,
}

impl AppointmentRequest {
    /// Create a new appointment request with validation.
    pub fn new(
        patient: Patient,
        priority: Priority,
        preferred_time: DateTime<Local>,
        reason: String,
        flexibility_minutes: i64,
    ) -> Result<Self, String> {
        if reason.is_empty() {
            return Err("Appointment reason cannot be empty".to_string());
        }
        if flexibility_minutes < 0 {
            return Err("Flexibility minutes cannot be negative".to_string());
        }

        Ok(AppointmentRequest {
            request_id: Uuid::new_v4().to_string(),
            patient,
            priority,
            preferred_time,
            reason,
            flexibility_minutes,
            created_at: Local::now(),
        })
    }

    /// Calculate the earliest acceptable appointment time.
    pub fn earliest_acceptable(&self) -> DateTime<Local> {
        self.preferred_time - Duration::minutes(self.flexibility_minutes)
    }

    /// Calculate the latest acceptable appointment time.
    pub fn latest_acceptable(&self) -> DateTime<Local> {
        self.preferred_time + Duration::minutes(self.flexibility_minutes)
    }

    /// Check if a time slot falls within the acceptable range.
    pub fn is_time_acceptable(&self, slot: &TimeSlot) -> bool {
        slot.start_time >= self.earliest_acceptable() && slot.start_time <= self.latest_acceptable()
    }
}

impl PartialEq for AppointmentRequest {
    fn eq(&self, other: &Self) -> bool {
        self.request_id == other.request_id
    }
}

impl Eq for AppointmentRequest {}

impl PartialOrd for AppointmentRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AppointmentRequest {
    /// Compare requests for priority queue ordering.
    ///
    /// Higher priority requests come first. For equal priorities,
    /// earlier requests are processed first.
    fn cmp(&self, other: &Self) -> Ordering {
        match other.priority.cmp(&self.priority) {
            Ordering::Equal => self.created_at.cmp(&other.created_at),
            other_ordering => other_ordering,
        }
    }
}

/// Factory function to create an appointment request.
pub fn create_appointment_request(
    patient_id: String,
    patient_name: String,
    patient_contact: String,
    priority: &str,
    preferred_time: DateTime<Local>,
    reason: String,
    flexibility_minutes: i64,
) -> Result<AppointmentRequest, String> {
    let patient = Patient::new(patient_id, patient_name, patient_contact)?;
    let priority_enum = Priority::from_string(priority)?;

    AppointmentRequest::new(
        patient,
        priority_enum,
        preferred_time,
        reason,
        flexibility_minutes,
    )
}
