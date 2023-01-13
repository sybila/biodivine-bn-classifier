const invoke = window.__TAURI__.invoke

/*
	Compute engine object maintains connection to the rust backend that will actually
	do the work for us.
*/
let ComputeEngine = {

	waitingForResult: false,
	_address: "native",
	_connected: true,
	_pingRepeatToken: undefined,

	// Open connection, taking up to date address from user input.
	// Callback is called upon first ping.
	openConnection(callback = undefined, address = undefined) {
		// Native compute engine is always available.
		if (callback !== undefined) {
			callback();		
		}		
	},

	getAddress() {
		return this._address;
	},

	// Return current connection status.
	isConnected() {
		// Native compute engine is always connected.
		return true;
	},
	
	// Force requests connection even when ping was not established (for situations
	// where this is the first call).
	getWitness(witness, callback, force = false) {		
		if (!force && !this.isConnected()) {
			callback("Compute engine not connected.");
			return undefined;
		} else {
			return this._backendRequest("/get_witness/"+witness, (e, r) => {
				if (callback !== undefined) {
					callback(e, r);
				}
			}, "GET");
		}
	},

	getTreeWitness(nodeId, callback, force = false) {
		if (!force && !this.isConnected()) {
			callback("Compute engine not connected.");
			return undefined;
		} else {
			return this._backendRequest("/get_tree_witness/"+nodeId, (e, r) => {
				if (callback !== undefined) {
					callback(e, r);
				}
			}, "GET");	
		}
	},

	getBifurcationTree(callback, force = false) {
		invoke('get_decision_tree')
			.then((response) => {
				console.log(response);
				callback(undefined, JSON.parse(response));
			})
			.catch((error) => callback(error, undefined));
	},

	autoExpandBifurcationTree(node, depth, callback) {
		invoke('auto_expand_tree', { "depth": parseInt(depth), "nodeId": parseInt(node) })
			.then((response) => callback(undefined, JSON.parse(response)))
			.catch((error) => callback(error, undefined));
	},

	getDecisionAttributes(node, callback) {
		invoke('get_decision_attributes', { "nodeId": parseInt(node) })
			.then((response) => callback(undefined, JSON.parse(response)))
			.catch((error) => callback(error, undefined));
	},

	applyTreePrecision(precision, callback) {
		invoke("set_tree_precision", { "precision": parseInt(precision) })
			.then((response) => callback(undefined, response))
			.catch((error) => callback(error, undefined));
	},

	getTreePrecision(callback) {
		invoke("get_tree_precision")
			.then((response) => callback(undefined, response))
			.catch((error) => callback(error, undefined));
	},

	selectDecisionAttribute(node, attr, callback) {
		invoke("apply_decision_attribute", { "nodeId": parseInt(node), "attributeId": parseInt(attr) })
			.then((response) => callback(undefined, JSON.parse(response)))
			.catch((error) => callback(error, undefined))
	},

	deleteDecision(nodeId, callback) {
		invoke("revert_decision", { "nodeId": parseInt(nodeId) })
			.then((response) => callback(undefined, JSON.parse(response)))
			.catch((error) => callback(error, undefined));
	},

}