use std::cell::{Ref, RefMut, RefCell};
use gtk::prelude::*;
use gtk::{self, gio, glib};
use gio::ListStore;
use std::thread;
use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use glib::clone;

use crate::vm_gobject::VMGObject;
use crate::client::client::*;
use crate::client::admin::ApplicationRequest;

pub mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct DataProvider {
        pub store: RefCell<ListStore>,
        pub status: bool,
        pub request_sender: Sender<ClientServiceRequest>,
        pub response_receiver: Receiver<ClientServiceResponse>,
    }

    impl DataProvider {
        pub fn new() -> Self {
            let init_store = ListStore::new::<VMGObject>();//Self::fill_by_mock_data();
            let (request_tx, response_rx): (Sender<ClientServiceRequest>, Receiver<ClientServiceResponse>) = Self::make_client_thread();

            Self {
                store: RefCell::new(init_store),
                status: false,
                request_sender: request_tx,
                response_receiver: response_rx,
            }
        }

        pub fn establish_connection(&self) {

        }

        fn fill_by_mock_data() -> ListStore {
            let init_store = ListStore::new::<VMGObject>();
            let mut vec: Vec<VMGObject> = Vec::new();
            vec.push(VMGObject::new("VM1".to_string(), String::from("This is the file.pdf")));
            vec.push(VMGObject::new("VM2".to_string(), String::from("Google Chrome")));
            init_store.extend_from_slice(&vec);
            return init_store;
        }

        fn make_client_thread() -> (Sender<ClientServiceRequest>, Receiver<ClientServiceResponse>) {
            let (request_tx, request_rx): (Sender<ClientServiceRequest>, Receiver<ClientServiceRequest>) = mpsc::channel();
            let (response_tx, response_rx): (Sender<ClientServiceResponse>, Receiver<ClientServiceResponse>) = mpsc::channel();
            let endpoint = String::from("http://[::1]:50051");

            thread::spawn(move || {
                client_service_thread(endpoint, request_rx, response_tx, Self::response_callback);
            });

            (request_tx, response_rx)
        }

        pub fn get_store_copy(&self) -> ListStore {
            self.store.borrow().clone()
        }

        pub fn get_store_ref(&self) -> Ref<ListStore> {
            self.store.borrow()
        }

        pub fn get_store_mut_ref(&self) -> RefMut<ListStore> {
            self.store.borrow_mut()
        }

        pub fn add_vm(&self, vm: VMGObject) {
            let mut store = self.store.borrow_mut();
            store.append(&vm);
        }

        pub fn update_request(&self) {
            println!("Update request...");
            self.request_sender.send(ClientServiceRequest::AppList()).expect("Send error");

            //blocks main thread!
            while let Ok(response) = self.response_receiver.recv() {
                match response {
                    ClientServiceResponse::AppList(app_list) => {
                        println!("List: {:?}", app_list.applications);
                        let mut store = self.store.borrow_mut();
                        store.remove_all();
                        let mut vec: Vec<VMGObject> = Vec::new();
                        for app in app_list.applications {
                            store.append(&VMGObject::new(app.name, app.description));
                        }
                        break;
                    },
                    _ => todo!(),
                }
            }//*/

            /*
            let response_receiver = Arc::new(self.response_receiver);//tokio::sunc::mpsc::channel is needed
            let mut store = self.store.borrow_mut();
            glib::spawn_future_local(async move {
                while let Ok(response) = response_receiver.recv() {
                    match response {
                        ClientServiceResponse::AppList(app_list) => {
                            println!("List: {:?}", app_list.applications);
                            
                            store.remove_all();
                            let mut vec: Vec<VMGObject> = Vec::new();
                            for app in app_list.applications {
                                store.append(&VMGObject::new(app.name, app.description));
                            }
                            break;
                        },
                        _ => todo!(),
                    }
                }
            });
            */
        }
        
        pub fn response_callback(response: ClientServiceResponse) {
            match response {
                ClientServiceResponse::AppList(app_list) => {
                    println!("Callback! List: {:?}", app_list.applications);
                    
                },
                _ => todo!(),
            }
        }
    }

    impl Default for DataProvider {
        fn default() -> Self {
            Self::new()
        }
    }
}

