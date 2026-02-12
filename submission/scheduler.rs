#![allow(dead_code)]
/// Appointment scheduling algorithm with priority-based scheduling.
///
/// This module provides the AppointmentScheduler struct which processes
/// appointment requests and schedules them efficiently based on priority
/// and time preferences.

use crate::calendar::DoctorCalendar;
use crate::models::{Appointment, AppointmentRequest, Patient, Priority, TimeSlot};
use chrono::{DateTime, Local};
use std::collections::BinaryHeap;
use uuid::Uuid;

/// Result of a scheduling attempt for a single request.
#[derive(Debug, Clone)]
pub struct SchedulingResult {
    pub request: AppointmentRequest,
    pub appointment: Option<Appointment>,
    pub success: bool,
    pub message: String,
}

/// Result of scheduling multiple requests.
#[derive(Debug)]
pub struct BatchSchedulingResult {
    pub confirmed: Vec<Appointment>,
    pub failed: Vec<SchedulingResult>,
    pub total_requests: usize,
}

impl BatchSchedulingResult {
    /// Calculate the success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.confirmed.len() as f64 / self.total_requests as f64) * 100.0
    }
}

/// Priority-based appointment scheduler.
///
/// This scheduler processes appointment requests using a priority queue,
/// ensuring that emergency and urgent appointments are scheduled before
/// routine ones. It attempts to schedule appointments as close to the
/// patient's preferred time as possible within their flexibility window.
pub struct AppointmentScheduler {
    pub calendar: DoctorCalendar,
    pub allow_fallback: bool,
    request_queue: BinaryHeap<AppointmentRequest>,
}

impl AppointmentScheduler {
    /// Initialize the scheduler.
    pub fn new(calendar: DoctorCalendar, allow_fallback: bool) -> Self {
        AppointmentScheduler {
            calendar,
            allow_fallback,
            request_queue: BinaryHeap::new(),
        }
    }

    /// Add a request to the scheduling queue.
    pub fn add_request(&mut self, request: AppointmentRequest) {
        self.request_queue.push(request);
    }

    /// Add multiple requests to the queue.
    pub fn add_requests(&mut self, requests: Vec<AppointmentRequest>) {
        for request in requests {
            self.add_request(request);
        }
    }

    /// Find the best available slot for a request.
    fn find_slot_for_request(&self, request: &AppointmentRequest) -> Option<TimeSlot> {
        let mut slot = self
            .calendar
            .find_available_slot(request.preferred_time, request.flexibility_minutes);

        if slot.is_none() && self.allow_fallback {
            slot = self.calendar.find_next_available_slot(request.preferred_time);
        }

        slot
    }

    /// Schedule a single appointment request.
    pub fn schedule_single(&mut self, request: AppointmentRequest) -> SchedulingResult {
        let slot = self.find_slot_for_request(&request);

        let slot = match slot {
            Some(s) => s,
            None => {
                return SchedulingResult {
                    request,
                    appointment: None,
                    success: false,
                    message: "No available time slots found".to_string(),
                };
            }
        };

        let was_preferred = request.is_time_acceptable(&slot);
        let preferred_time = request.preferred_time;
        
        // Clone what we need to keep for the return value
        let request_id = request.request_id.clone();
        let patient = request.patient.clone();
        let priority = request.priority;
        let reason = request.reason.clone();
        let flexibility_minutes = request.flexibility_minutes;
        let created_at = request.created_at;

        match self.calendar.book_slot(
            &slot,
            request.patient,  // Move into book_slot
            request.priority,
            request.reason,   // Move into book_slot
        ) {
            Ok(appointment) => {
                let message = if was_preferred {
                    format!(
                        "Scheduled at preferred time: {}",
                        slot.start_time.format("%Y-%m-%d %H:%M")
                    )
                } else {
                    format!(
                        "Scheduled at alternative time: {} (preferred was {})",
                        slot.start_time.format("%Y-%m-%d %H:%M"),
                        preferred_time.format("%H:%M")
                    )
                };

                // Reconstruct request for return value
                let returned_request = AppointmentRequest {
                    request_id,
                    patient,
                    priority,
                    preferred_time,
                    reason,
                    flexibility_minutes,
                    created_at,
                };

                SchedulingResult {
                    request: returned_request,
                    appointment: Some(appointment),
                    success: true,
                    message,
                }
            }
            Err(e) => {
                // Reconstruct request for error case
                let returned_request = AppointmentRequest {
                    request_id,
                    patient,
                    priority,
                    preferred_time,
                    reason,
                    flexibility_minutes,
                    created_at,
                };

                SchedulingResult {
                    request: returned_request,
                    appointment: None,
                    success: false,
                    message: e,
                }
            }
        }
    }

    /// Process all requests in the queue by priority.
    pub fn process_queue(&mut self) -> BatchSchedulingResult {
        let mut confirmed = Vec::new();
        let mut failed = Vec::new();
        let total = self.request_queue.len();

        while let Some(request) = self.request_queue.pop() {
            let result = self.schedule_single(request);

            if result.success {
                if let Some(appointment) = result.appointment {
                    confirmed.push(appointment);
                }
            } else {
                failed.push(result);
            }
        }

        BatchSchedulingResult {
            confirmed,
            failed,
            total_requests: total,
        }
    }

    /// Schedule a batch of requests in priority order.
    pub fn schedule_batch(&mut self, requests: Vec<AppointmentRequest>) -> BatchSchedulingResult {
        self.add_requests(requests);
        self.process_queue()
    }

    /// Reschedule an existing appointment to a new time.
    pub fn reschedule_appointment(
        &mut self,
        appointment_id: &str,
        new_preferred_time: DateTime<Local>,
        flexibility_minutes: i64,
    ) -> SchedulingResult {
        // Get the original appointment or return early if not found
        let appointment = match self.calendar.get_appointment_by_id(appointment_id) {
            Some(apt) => apt,
            None => {
                // Create a minimal error response without panicking
                return SchedulingResult {
                    request: AppointmentRequest {
                        request_id: Uuid::new_v4().to_string(),
                        patient: Patient {
                            patient_id: "unknown".to_string(),
                            name: "Unknown".to_string(),
                            contact: "unknown".to_string(),
                        },
                        priority: Priority::Routine,
                        preferred_time: new_preferred_time,
                        reason: "Reschedule".to_string(),
                        flexibility_minutes,
                        created_at: Local::now(),
                    },
                    appointment: None,
                    success: false,
                    message: "Original appointment not found".to_string(),
                };
            }
        };

        // Build the reschedule request once
        let reschedule_request = AppointmentRequest {
            request_id: Uuid::new_v4().to_string(),
            patient: appointment.patient.clone(),
            priority: appointment.priority,
            preferred_time: new_preferred_time,
            reason: appointment.reason.clone(),
            flexibility_minutes,
            created_at: Local::now(),
        };

        let new_slot = self
            .calendar
            .find_available_slot(new_preferred_time, flexibility_minutes);

        let new_slot = match new_slot {
            Some(slot) => slot,
            None => {
                return SchedulingResult {
                    request: reschedule_request,
                    appointment: None,
                    success: false,
                    message: "No available slots at the requested time".to_string(),
                };
            }
        };

        // Cancel old, book new
        self.calendar.cancel_appointment(appointment_id);
        
        match self.calendar.book_slot(
            &new_slot,
            appointment.patient,  // Move, don't clone
            appointment.priority,
            appointment.reason,   // Move, don't clone
        ) {
            Ok(new_appointment) => SchedulingResult {
                request: reschedule_request,
                appointment: Some(new_appointment),
                success: true,
                message: format!(
                    "Rescheduled to {}",
                    new_slot.start_time.format("%Y-%m-%d %H:%M")
                ),
            },
            Err(e) => SchedulingResult {
                request: reschedule_request,
                appointment: None,
                success: false,
                message: format!("Failed to reschedule: {}", e),
            },
        }
    }

    /// Get the number of pending requests in the queue.
    pub fn get_pending_count(&self) -> usize {
        self.request_queue.len()
    }

    /// Clear all pending requests from the queue.
    pub fn clear_queue(&mut self) -> usize {
        let count = self.request_queue.len();
        self.request_queue.clear();
        count
    }
}
