title: DXN


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


_: **2. Server initialization**

Client --> Server: Load Server setup flow
Server --> Client: [Webhook] **Server.setup.status=inprogress**

Client --> Server: Load Server keyvault flow
Server --> Client: [Webhook] **Server.keyvault.status=complete**

Client --> Server: Load Server encryption flow
Server --> Client: [Webhook] **Server.encryption.status=complete**

Client --> Server: Load Server data flow
Server --> Client: [Webhook] **Server.data.status=complete**

Client --> Server: Load Server functions flow
Server --> Client: [Webhook] **Server.functions.status=complete**

Client --> Server: Load Server services flow
Server --> Client: [Webhook] **Server.services.status=complete**

Client -> Server: Load Server page


_: **3. Server load**

Client -> Client: Load cached server
Client --> Server: Load actual server
Server --> Client: [Webhook] server.updatedAt > cached.updatedAt
Client -> Client: Update cache



_: **4. App / Server sync**

Client -> Client: OnSync timeout
Client -> Portal: Fetch latest events
Client -x Client: Run aggregates



_: **5. Server termination sync**

Client -> Portal: Kill server
Portal -> Server: Decomission
Portal -> Cloud Provider: Remove Virtual instance
Portal -> Manifest: Remove client/server uuid pairing
Portal -> Portal: Decomission server
Note:
1. Delete all projects
2. Remove virtual instance with provider
3. Test connection
Portal --> Client: [Webhook] **Server.status=decomissioned**
Portal -> Portal DB: Update Server entry