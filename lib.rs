#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod todo {

    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;

    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Task {
        pub id: u32,
        pub description: String,
        pub completed: bool,
    }

    #[ink(storage)]
    pub struct Todo {
        tasks: Vec<Task>,
        next_id: u32,
    }

    impl Default for Todo {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Todo {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                tasks: Vec::new(),
                next_id: 0,
            }
        }

        #[ink(message)]
        pub fn add_task(&mut self, description: String) -> u32 {
            let id = self.next_id;
            let task = Task {
                id,
                description,
                completed: false,
            };
            self.tasks.push(task);
            self.next_id = self.next_id.saturating_add(1);
            id
        }

        #[ink(message)]
        pub fn complete_task(&mut self, id: u32) -> bool {
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                task.completed = true;
                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn remove_task(&mut self, id: u32) -> bool {
            if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
                self.tasks.remove(pos);
                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn get_tasks(&self) -> Vec<Task> {
            self.tasks.clone()
        }

        #[ink(message)]
        pub fn get_task(&self, id: u32) -> Option<Task> {
            self.tasks.iter().find(|t| t.id == id).cloned()
        }
    }

    /// Unit tests in Rust are normally defined within such a #[cfg(test)]
    /// module and test functions are marked with a #[test] attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_works() {
            let todo = Todo::new();
            assert_eq!(todo.get_tasks().len(), 0);
        }

        #[ink::test]
        fn add_task_works() {
            let mut todo = Todo::new();
            let id = todo.add_task(String::from("Buy milk"));
            assert_eq!(id, 0);
            assert_eq!(todo.get_tasks().len(), 1);

            let task = todo.get_task(id).unwrap();
            assert_eq!(task.description, "Buy milk");
            assert_eq!(task.completed, false);
        }

        #[ink::test]
        fn complete_task_works() {
            let mut todo = Todo::new();
            let id = todo.add_task(String::from("Buy milk"));

            assert!(todo.complete_task(id));
            let task = todo.get_task(id).unwrap();
            assert_eq!(task.completed, true);
        }

        #[ink::test]
        fn remove_task_works() {
            let mut todo = Todo::new();
            let id = todo.add_task(String::from("Buy milk"));

            assert!(todo.remove_task(id));
            assert_eq!(todo.get_tasks().len(), 0);
            assert!(todo.get_task(id).is_none());
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the e2e-tests feature flag enabled (--features e2e-tests)
    /// - Are running a Substrate node which contains pallet-contracts in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test Result type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_add_and_complete_task(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let mut constructor = TodoRef::new();
            let contract = client
                .instantiate("todo", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Todo>();

            // Add a task
            let add_task = call_builder.add_task(String::from("Test task"));
            let add_result = client
                .call(&ink_e2e::alice(), &add_task)
                .submit()
                .await
                .expect("add_task failed");
            let task_id = add_result.return_value();

            // Get tasks
            let get_tasks = call_builder.get_tasks();
            let tasks_result = client.call(&ink_e2e::alice(), &get_tasks).dry_run().await?;
            assert_eq!(tasks_result.return_value().len(), 1);

            // Complete task
            let complete = call_builder.complete_task(task_id);
            let _complete_result = client
                .call(&ink_e2e::alice(), &complete)
                .submit()
                .await
                .expect("complete_task failed");

            // Verify task is completed
            let get_task = call_builder.get_task(task_id);
            let task_result = client.call(&ink_e2e::alice(), &get_task).dry_run().await?;
            assert!(task_result.return_value().unwrap().completed);

            Ok(())
        }
    }
}