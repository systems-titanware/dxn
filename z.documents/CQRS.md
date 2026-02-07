title: CQRS

List domain models to data models to enable CQRS for those models
Uses pre-built functions to integrate CQRS to data models

Commands
1. https://doc.rust-cqrs.org/intro_add_commands.html
2. CreateCommand, ReadCommand, UpdateCommand, DeleteCommand

Events
1. https://doc.rust-cqrs.org/intro_add_events.html
2. List of events in config
1.1. events.domain: [ "DepositMoney", "TransferMoney", "WithdrawMoney" ]
1.2. events.defaults: [ "Create", "Read", "Updated", "Delete" ]

Aggregate
2. DomainModel

Error
2. DomainModelError

Service
2. DomainModelService

Queries
2. DomainModelQuery

EventStore
1. https://doc.rust-cqrs.org/demo_event_store.html

_: **1. App initialization**
Client -> Portal: Create Server
Portal -> Server: Add client/server mapping/manifest
Portal -> Portal DB: Add Server entry
Portal -> Manifest: Update manifest
Portal <-> Manifest: Confirm update

Portal -> Server: Initialize
Portal -> Cloud Provider: Setup Virtual instance
Portal -> Portal: Configure server
Note:
1. Run build/test/run for all projects
2. Deploy to virtual instance
3. Test connection
Portal --> Client: [Webhook] **Server.status=ready**

Client --> Server: Load Server setup flow
Server --> Client: [Webhook] **Server.setup.status=inprogress**
