#![allow(dead_code)]
/// Calendar management for the appointment scheduling system.
///
/// This module provides the DoctorCalendar class which manages available
/// time slots and booked appointments for a doctor's schedule.

use crate::models::{Appointment, Patient, Priority, TimeSlot};
use chrono::{DateTime, Datelike, Duration, Local, NaiveTime};  // REMOVED Timelike (unused), ADDED Datelike
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]  // ADDED: Derive Clone instead of manual impl in main.rs
pub struct DoctorCalendar {
    pub doctor_name: String,
    pub doctor_id: String,
    pub default_slot_duration: i64,
    time_slots: HashMap<String, TimeSlot>,
    appointments: HashMap<String, Appointment>,
}

impl DoctorCalendar {
    /// Initialize a new doctor calendar.
    pub fn new(doctor_name: String, default_slot_duration: i64) -> Result<Self, String> {
        if doctor_name.is_empty() {
            return Err("Doctor name cannot be empty".to_string());
        }
        if default_slot_duration <= 0 {
            return Err("Slot duration must be positive".to_string());
        }

        Ok(DoctorCalendar {
            doctor_name,
            doctor_id: Uuid::new_v4().to_string(),
            default_slot_duration,
            time_slots: HashMap::new(),
            appointments: HashMap::new(),
        })
    }

    /// Get all time slots sorted by start time.
    pub fn time_slots(&self) -> Vec<TimeSlot> {
        let mut slots: Vec<TimeSlot> = self.time_slots.values().cloned().collect();
        slots.sort_by_key(|s| s.start_time);
        slots
    }

    /// Get all available (unbooked) time slots.
    pub fn available_slots(&self) -> Vec<TimeSlot> {
        let mut slots: Vec<TimeSlot> = self
            .time_slots
            .values()
            .filter(|s| s.is_available)
            .cloned()
            .collect();
        slots.sort_by_key(|s| s.start_time);
        slots
    }

    /// Get all confirmed appointments sorted by time.
    pub fn appointments(&self) -> Vec<Appointment> {
        let mut appointments: Vec<Appointment> = self.appointments.values().cloned().collect();
        appointments.sort_by_key(|a| a.time_slot.start_time);
        appointments
    }

    /// Add a time slot to the calendar.
    pub fn add_time_slot(&mut self, slot: TimeSlot) -> Result<(), String> {
        for existing in self.time_slots.values() {
            if slot.overlaps_with(existing) {
                return Err(format!(
                    "Time slot overlaps with existing slot: {} - {}",
                    existing.start_time.format("%Y-%m-%d %H:%M"),
                    existing.end_time.format("%Y-%m-%d %H:%M")
                ));
            }
        }
        self.time_slots.insert(slot.slot_id.clone(), slot);
        Ok(())
    }

    /// Remove a time slot from the calendar.
    pub fn remove_time_slot(&mut self, slot_id: &str) -> bool {
        self.time_slots.remove(slot_id).is_some()
    }

    /// Generate time slots for a single day.
    pub fn generate_daily_slots(
        &mut self,
        date: DateTime<Local>,
        start_hour: u32,
        end_hour: u32,
        slot_duration_minutes: Option<i64>,
        break_start: Option<NaiveTime>,
        break_end: Option<NaiveTime>,
    ) -> Vec<TimeSlot> {
        let duration = slot_duration_minutes.unwrap_or(self.default_slot_duration);
        let mut slots = Vec::new();

        let mut current = date
            .date_naive()
            .and_hms_opt(start_hour, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();

        let end = date
            .date_naive()
            .and_hms_opt(end_hour, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();

        while current + Duration::minutes(duration) <= end {
            let slot_end = current + Duration::minutes(duration);

            let mut skip = false;
            if let (Some(break_start), Some(break_end)) = (break_start, break_end) {
                let slot_start_time = current.time();
                let slot_end_time = slot_end.time();

                if slot_start_time < break_end && slot_end_time > break_start {
                    skip = true;
                }
            }

            if !skip {
                if let Ok(slot) = TimeSlot::new(current, slot_end) {
                    if self.add_time_slot(slot.clone()).is_ok() {
                        slots.push(slot);
                    }
                }
            }

            current = slot_end;
        }

        slots
    }

    /// Generate time slots for multiple weeks.
    pub fn generate_weekly_slots(
        &mut self,
        start_date: DateTime<Local>,
        weeks: usize,
        working_days: Option<Vec<u32>>,
        start_hour: u32,
        end_hour: u32,
        slot_duration_minutes: Option<i64>,
        break_start: Option<NaiveTime>,
        break_end: Option<NaiveTime>,
    ) -> Vec<TimeSlot> {
        let working_days = working_days.unwrap_or_else(|| vec![0, 1, 2, 3, 4]);
        let mut all_slots = Vec::new();
        let mut current_date = start_date;

        for _ in 0..(weeks * 7) {
            // FIXED: Datelike trait now in scope
            if working_days.contains(&current_date.weekday().num_days_from_monday()) {
                let slots = self.generate_daily_slots(
                    current_date,
                    start_hour,
                    end_hour,
                    slot_duration_minutes,
                    break_start,
                    break_end,
                );
                all_slots.extend(slots);
            }
            current_date = current_date + Duration::days(1);
        }

        all_slots
    }

    /// Find an available slot near the preferred time.
    pub fn find_available_slot(
        &self,
        preferred_time: DateTime<Local>,
        flexibility_minutes: i64,
    ) -> Option<TimeSlot> {
        let earliest = preferred_time - Duration::minutes(flexibility_minutes);
        let latest = preferred_time + Duration::minutes(flexibility_minutes);

        // FIXED: Borrow checker error E0716
        // Changed from storing references to storing owned values
        let slots = self.available_slots();  // Owned Vec<TimeSlot>, lives for the whole scope
        
        let mut candidates: Vec<&TimeSlot> = slots
            .iter()
            .filter(|slot| slot.start_time >= earliest && slot.start_time <= latest)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        candidates.sort_by_key(|s| {
            (s.start_time - preferred_time)
                .num_seconds()
                .abs()
        });

        candidates.first().map(|&s| s.clone())
    }

    /// Find the next available slot after a given time.
    pub fn find_next_available_slot(&self, after: DateTime<Local>) -> Option<TimeSlot> {
        // FIXED: Changed into_iter() to iter() and cloned()
        // into_iter() would consume self.available_slots(), iter() borrows
        self.available_slots()
            .iter()
            .find(|slot| slot.start_time >= after)
            .cloned()
    }

    /// Find all available slots on a specific date.
    pub fn find_available_slots_on_date(&self, date: DateTime<Local>) -> Vec<TimeSlot> {
        // FIXED: Same pattern - use iter() not into_iter()
        self.available_slots()
            .iter()
            .filter(|slot| slot.start_time.date_naive() == date.date_naive())
            .cloned()
            .collect()
    }

    /// Book a time slot for a patient.
    pub fn book_slot(
        &mut self,
        slot: &TimeSlot,
        patient: Patient,
        priority: Priority,
        reason: String,
    ) -> Result<Appointment, String> {
        let stored_slot = self
            .time_slots
            .get_mut(&slot.slot_id)
            .ok_or("Time slot not found in calendar")?;

        if !stored_slot.is_available {
            return Err("Time slot is not available".to_string());
        }

        stored_slot.is_available = false;

        let appointment = Appointment::new(patient, stored_slot.clone(), priority, reason)?;
        self.appointments
            .insert(appointment.appointment_id.clone(), appointment.clone());

        Ok(appointment)
    }

    /// Cancel an appointment and free up the time slot.
    pub fn cancel_appointment(&mut self, appointment_id: &str) -> bool {
        if let Some(appointment) = self.appointments.remove(appointment_id) {
            if let Some(slot) = self.time_slots.get_mut(&appointment.time_slot.slot_id) {
                slot.is_available = true;
            }
            true
        } else {
            false
        }
    }

    /// Get all appointments on a specific date.
    pub fn get_appointments_on_date(&self, date: DateTime<Local>) -> Vec<Appointment> {
        // FIXED: appointments() returns Vec<Appointment>, not a reference
        self.appointments()
            .into_iter()
            .filter(|apt| apt.time_slot.start_time.date_naive() == date.date_naive())
            .collect()
    }

    /// Get an appointment by its ID.
    pub fn get_appointment_by_id(&self, appointment_id: &str) -> Option<Appointment> {
        self.appointments.get(appointment_id).cloned()
    }
}

impl std::fmt::Display for DoctorCalendar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DoctorCalendar({}, slots={}, appointments={})",
            self.doctor_name,
            self.time_slots.len(),
            self.appointments.len()
        )
    }
}
