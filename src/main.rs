#![allow(dead_code)]
/// Command-line interface for the appointment scheduling system.
///
/// This module provides an interactive CLI for managing doctor calendars,
/// submitting appointment requests, and viewing scheduled appointments.

mod calendar;
mod models;
mod scheduler;

use calendar::DoctorCalendar;
use chrono::{Datelike, Duration, Local, NaiveTime};  // Added Datelike
use models::create_appointment_request;  // Removed Priority (unused)
use scheduler::AppointmentScheduler;
use std::io::{self, Write};

struct AppointmentCLI {
    calendar: Option<DoctorCalendar>,
    scheduler: Option<AppointmentScheduler>,
    running: bool,
}

impl AppointmentCLI {
    fn new() -> Self {
        AppointmentCLI {
            calendar: None,
            scheduler: None,
            running: true,
        }
    }

    fn print_header(&self) {
        println!("\n{}", "=".repeat(60));
        println!("       APPOINTMENT SCHEDULING SYSTEM");
        println!("{}", "=".repeat(60));
    }

    fn print_menu(&self) {
        println!("\n--- Main Menu ---");
        println!("1. Setup doctor calendar");
        println!("2. Generate time slots");
        println!("3. Submit appointment request");
        println!("4. Process all requests");
        println!("5. View available slots");
        println!("6. View confirmed appointments");
        println!("7. Cancel appointment");
        println!("8. Run demo");
        println!("9. Exit");
        println!("{}", "-".repeat(20));
    }

    fn get_input(&self, prompt: &str, default: Option<&str>) -> String {
        if let Some(def) = default {
            print!("{} [{}]: ", prompt, def);
        } else {
            print!("{}: ", prompt);
        }
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            default.unwrap_or("").to_string()
        } else {
            input.to_string()
        }
    }

    fn get_int_input(&self, prompt: &str, default: Option<i32>) -> i32 {
        loop {
            let default_str = default.map(|d| d.to_string());
            let input = self.get_input(prompt, default_str.as_deref());

            if let Ok(value) = input.parse::<i32>() {
                return value;
            }
            println!("Please enter a valid number");
        }
    }

    fn setup_calendar(&mut self) {
        println!("\n--- Setup Doctor Calendar ---");

        let doctor_name = self.get_input("Doctor name", Some("Dr. Smith"));
        let slot_duration = self.get_int_input("Default appointment duration (minutes)", Some(30));

        match DoctorCalendar::new(doctor_name.clone(), slot_duration as i64) {
            Ok(calendar) => {
                let scheduler = AppointmentScheduler::new(calendar.clone(), true);
                self.calendar = Some(calendar);
                self.scheduler = Some(scheduler);

                println!("\nCalendar created for {}", doctor_name);
                println!("Default slot duration: {} minutes", slot_duration);
            }
            Err(e) => println!("Error creating calendar: {}", e),
        }
    }

    fn generate_slots(&mut self) {
        if self.calendar.is_none() {
            println!("\nPlease setup a calendar first (option 1)");
            return;
        }

        println!("\n--- Generate Time Slots ---");

        let days = self.get_int_input("Number of days", Some(5));
        let start_hour = self.get_int_input("Working hours start", Some(9)) as u32;
        let end_hour = self.get_int_input("Working hours end", Some(17)) as u32;

        let include_break = self.get_input("Include lunch break? (y/n)", Some("y"));

        let (break_start, break_end) = if include_break.to_lowercase() == "y" {
            println!("Lunch break: 12:00 - 13:00");
            (
                Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            )
        } else {
            (None, None)
        };

        let mut total_slots = 0;
        let mut current_date = Local::now() + Duration::days(1);

        // Fixed: Datelike trait is now in scope via use chrono::Datelike
        if let Some(mut calendar) = self.calendar.take() {
            for _ in 0..days {
                if current_date.weekday().num_days_from_monday() < 5 {
                    let slots = calendar.generate_daily_slots(
                        current_date,
                        start_hour,
                        end_hour,
                        None,
                        break_start,
                        break_end,
                    );
                    total_slots += slots.len();
                }
                current_date = current_date + Duration::days(1);
            }

            println!("\nGenerated {} time slots", total_slots);

            // Create new scheduler with updated calendar
            let new_scheduler = AppointmentScheduler::new(calendar.clone(), true);
            self.calendar = Some(calendar);
            self.scheduler = Some(new_scheduler);
        }
    }

    fn submit_request(&mut self) {
        if self.scheduler.is_none() {
            println!("\nPlease setup a calendar first (option 1)");
            return;
        }

        println!("\n--- Submit Appointment Request ---");

        let patient_name = self.get_input("Patient name", None);
        let patient_contact = self.get_input("Patient contact (phone/email)", None);
        let reason = self.get_input("Reason for appointment", None);

        println!("\nPriority levels:");
        println!("  1. Routine");
        println!("  2. Urgent");
        println!("  3. Emergency");
        let priority_choice = self.get_int_input("Select priority", Some(1));

        let priority = match priority_choice {
            1 => "routine",
            2 => "urgent",
            3 => "emergency",
            _ => "routine",
        };

        println!("\nPreferred time (tomorrow at 10:00 AM as default)");
        let hours = self.get_int_input("Hour (0-23)", Some(10));
        let minutes = self.get_int_input("Minute (0-59)", Some(0));

        let preferred_time = (Local::now() + Duration::days(1))
            .date_naive()
            .and_hms_opt(hours as u32, minutes as u32, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();

        let flexibility = self.get_int_input("Time flexibility (minutes)", Some(60)) as i64;

        let patient_id = uuid::Uuid::new_v4().to_string()[..8].to_string();

        match create_appointment_request(
            patient_id,
            patient_name.clone(),
            patient_contact,
            priority,
            preferred_time,
            reason,
            flexibility,
        ) {
            Ok(request) => {
                if let Some(scheduler) = &mut self.scheduler {
                    scheduler.add_request(request);
                    println!("\nRequest submitted for {}", patient_name);
                    println!("Priority: {}", priority.to_uppercase());
                    println!(
                        "Preferred time: {}",
                        preferred_time.format("%Y-%m-%d %H:%M")
                    );
                    println!("Pending requests in queue: {}", scheduler.get_pending_count());
                }
            }
            Err(e) => println!("Error creating request: {}", e),
        }
    }

    fn process_requests(&mut self) {
        if self.scheduler.is_none() {
            println!("\nPlease setup a calendar first (option 1)");
            return;
        }

        let pending = self.scheduler.as_ref().unwrap().get_pending_count();
        if pending == 0 {
            println!("\nNo pending requests to process");
            return;
        }

        println!("\n--- Processing {} requests ---", pending);
        
        // Take ownership temporarily
        let mut scheduler = self.scheduler.take().unwrap();
        let result = scheduler.process_queue();

        println!("\n--- Scheduling Results ---");
        println!("  Total requests: {}", result.total_requests);
        println!("  Confirmed: {}", result.confirmed.len());
        println!("  Failed: {}", result.failed.len());
        println!("  Success rate: {:.1}%", result.success_rate());

        if !result.confirmed.is_empty() {
            println!("\nConfirmed appointments:");
            for apt in &result.confirmed {
                println!(
                    "  - {}: {} ({})",
                    apt.patient.name,
                    apt.time_slot.start_time.format("%Y-%m-%d %H:%M"),
                    apt.priority.name()
                );
            }
        }

        if !result.failed.is_empty() {
            println!("\nFailed requests:");
            for fail in &result.failed {
                println!("  - {}: {}", fail.request.patient.name, fail.message);
            }
        }

        // Put back the scheduler and update calendar
        self.calendar = Some(scheduler.calendar.clone());
        self.scheduler = Some(scheduler);
    }

    fn view_available_slots(&self) {
        if self.calendar.is_none() {
            println!("\nPlease setup a calendar first (option 1)");
            return;
        }

        if let Some(calendar) = &self.calendar {
            // Fixed: Using getter method, not direct field access
            let slots = calendar.available_slots();

            if slots.is_empty() {
                println!("\nNo available time slots");
                return;
            }

            println!("\n--- Available Time Slots ({} total) ---", slots.len());

            let max_display = 20;
            let mut current_date = None;

            for (i, slot) in slots.iter().enumerate() {
                if i >= max_display {
                    println!("\n... and {} more slots", slots.len() - max_display);
                    break;
                }

                let slot_date = slot.start_time.date_naive();
                if Some(slot_date) != current_date {
                    current_date = Some(slot_date);
                    println!("\n{}:", slot_date.format("%A, %Y-%m-%d"));
                }

                println!(
                    "  {} - {}",
                    slot.start_time.format("%H:%M"),
                    slot.end_time.format("%H:%M")
                );
            }
        }
    }

    fn view_appointments(&self) {
        if self.calendar.is_none() {
            println!("\nPlease setup a calendar first (option 1)");
            return;
        }

        if let Some(calendar) = &self.calendar {
            // Fixed: Using getter method, not direct field access
            let appointments = calendar.appointments();

            if appointments.is_empty() {
                println!("\nNo confirmed appointments");
                return;
            }

            println!("\n--- Confirmed Appointments ({}) ---", appointments.len());

            let mut current_date = None;
            for apt in appointments {
                let apt_date = apt.time_slot.start_time.date_naive();
                if Some(apt_date) != current_date {
                    current_date = Some(apt_date);
                    println!("\n{}:", apt_date.format("%A, %Y-%m-%d"));
                }

                println!(
                    "  {} - {} ({}) - {}",
                    apt.time_slot.start_time.format("%H:%M"),
                    apt.patient.name,
                    apt.priority.name(),
                    apt.reason
                );
                println!("    ID: {}...", &apt.appointment_id[..8]);
            }
        }
    }

    fn cancel_appointment(&mut self) {
        if self.calendar.is_none() {
            println!("\nPlease setup a calendar first (option 1)");
            return;
        }

        if let Some(calendar) = &self.calendar {
            // Fixed: Using getter method
            let appointments = calendar.appointments();
            if appointments.is_empty() {
                println!("\nNo appointments to cancel");
                return;
            }

            println!("\n--- Cancel Appointment ---");
            println!("\nCurrent appointments:");
            for (i, apt) in appointments.iter().enumerate() {
                println!(
                    "  {}. {} - {}",
                    i + 1,
                    apt.patient.name,
                    apt.time_slot.start_time.format("%Y-%m-%d %H:%M")
                );
            }

            let choice = self.get_int_input("Select appointment to cancel (0 to go back)", Some(0));

            if choice == 0 {
                return;
            }

            if choice > 0 && (choice as usize) <= appointments.len() {
                let apt_to_cancel = &appointments[choice as usize - 1];
                let apt_id = apt_to_cancel.appointment_id.clone();
                let patient_name = apt_to_cancel.patient.name.clone();

                if let Some(calendar) = &mut self.calendar {
                    if calendar.cancel_appointment(&apt_id) {
                        println!("\nAppointment for {} cancelled", patient_name);
                        println!("Time slot is now available again");

                        // Update scheduler
                        if let Some(scheduler) = &mut self.scheduler {
                            scheduler.calendar = calendar.clone();
                        }
                    } else {
                        println!("\nFailed to cancel appointment");
                    }
                }
            }
        }
    }

    fn run_demo(&mut self) {
        println!("\n--- Running Demo ---");

        let calendar = DoctorCalendar::new("Dr. Demo".to_string(), 30).unwrap();
        let mut scheduler = AppointmentScheduler::new(calendar, true);

        let tomorrow = Local::now() + Duration::days(1);
        // Fixed: Datelike trait in scope
        scheduler.calendar.generate_daily_slots(
            tomorrow,
            9,
            17,
            None,
            Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
        );

        println!(
            "Created calendar with {} slots",
            scheduler.calendar.available_slots().len()
        );

        let requests = vec![
            create_appointment_request(
                "P001".to_string(),
                "John Smith".to_string(),
                "john@email.com".to_string(),
                "routine",
                tomorrow
                    .date_naive()
                    .and_hms_opt(10, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap(),
                "Annual checkup".to_string(),
                60,
            )
            .unwrap(),
            create_appointment_request(
                "P002".to_string(),
                "Jane Doe".to_string(),
                "jane@email.com".to_string(),
                "emergency",
                tomorrow
                    .date_naive()
                    .and_hms_opt(10, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap(),
                "Severe chest pain".to_string(),
                30,
            )
            .unwrap(),
            create_appointment_request(
                "P003".to_string(),
                "Bob Wilson".to_string(),
                "bob@email.com".to_string(),
                "urgent",
                tomorrow
                    .date_naive()
                    .and_hms_opt(14, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap(),
                "Follow-up on test results".to_string(),
                60,
            )
            .unwrap(),
            create_appointment_request(
                "P004".to_string(),
                "Alice Brown".to_string(),
                "alice@email.com".to_string(),
                "routine",
                tomorrow
                    .date_naive()
                    .and_hms_opt(11, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap(),
                "Prescription renewal".to_string(),
                120,
            )
            .unwrap(),
        ];

        println!("\nSubmitting {} appointment requests...", requests.len());
        println!("  - John Smith: Routine at 10:00");
        println!("  - Jane Doe: EMERGENCY at 10:00");
        println!("  - Bob Wilson: Urgent at 14:00");
        println!("  - Alice Brown: Routine at 11:00");

        let result = scheduler.schedule_batch(requests);

        println!("\n--- Scheduling Results ---");
        println!("Success rate: {:.1}%", result.success_rate());
        println!("\nConfirmed appointments (in scheduled order):");

        for apt in &result.confirmed {
            println!(
                "  [{:9}] {:15} -> {}",
                apt.priority.name(),
                apt.patient.name,
                apt.time_slot.start_time.format("%H:%M")
            );
        }

        println!("\nNote: Emergency patient Jane Doe was scheduled first,");
        println!("even though routine patient John Smith requested the same time.");

        // Store the results
        self.calendar = Some(scheduler.calendar.clone());
        self.scheduler = Some(scheduler);
    }

    fn run(&mut self) {
        self.print_header();

        while self.running {
            self.print_menu();

            let choice = self.get_int_input("Enter choice", Some(8));

            match choice {
                1 => self.setup_calendar(),
                2 => self.generate_slots(),
                3 => self.submit_request(),
                4 => self.process_requests(),
                5 => self.view_available_slots(),
                6 => self.view_appointments(),
                7 => self.cancel_appointment(),
                8 => self.run_demo(),
                9 => {
                    self.running = false;
                    println!("\nGoodbye!");
                }
                _ => println!("Invalid choice"),
            }
        }
    }
}

fn main() {
    let mut cli = AppointmentCLI::new();
    cli.run();
}
