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

	getTreeWitness(nodeId, randomize = false, callback) {
		// The response is forwarded "as is", because it is an AEON file.
		invoke('get_witness', { "nodeId": parseInt(nodeId), "randomize": randomize })
			.then((response) => callback(undefined, response))
			.catch((error) => callback(error, undefined));
	},

	getTreeWitnesses(num_witnesses, nodeId, randomize = false, callback) {
		console.log({ "numWitnesses": num_witnesses, "nodeId": parseInt(nodeId), "randomize": randomize });
		invoke('get_n_witnesses', { "numWitnesses": num_witnesses, "nodeId": parseInt(nodeId), "randomize": randomize })
			.then((response) => {
				console.log("Generating " + response.length + " witnesses.")
				callback(undefined, response);
			})
			.catch((error) => callback(error, undefined));
	},

	getNumNodeNetworks(nodeId, callback) {
		invoke('get_num_node_networks', { "nodeId": parseInt(nodeId)})
			.then((response) => callback(undefined, response))
			.catch((error) => callback(error, undefined));
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