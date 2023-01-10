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
		if (!force && !this.isConnected()) {
			callback("Compute engine not connected.");
			return undefined;
		} else {
			return this._backendRequest("/get_bifurcation_tree/", (e, r) => {
				if (callback !== undefined) {
					callback(e, r);
				}
			}, "GET");
		}
	},

	autoExpandBifurcationTree(node, depth, callback) {
		return this._backendRequest("/auto_expand/"+node+"/"+depth, (e, r) => {
			if (callback !== undefined) {
				callback(e, r);
			}
		}, "POST");
	},

	getDecisionAttributes(node, callback) {
		return this._backendRequest("/get_attributes/"+node, (e, r) => {
			if (callback !== undefined) {
				callback(e, r);
			}
		});
	},

	applyTreePrecision(precision, callback) {
		invoke("set_tree_precision", { "precision": parseInt(precision) })
			.then((response) => {	
				console.log(response);			
				callback(undefined, response)
			});
	},

	getTreePrecision(callback) {
		invoke("get_tree_precision")
			.then((response) => {				
				callback(undefined, response)
			});		
	},

	selectDecisionAttribute(node, attr, callback) {
		return this._backendRequest("/apply_attribute/"+node+"/"+attr, (e, r) => {
			if (callback !== undefined) {
				callback(e, r);
			}
		}, "POST");
	},

	deleteDecision(nodeId, callback) {
		return this._backendRequest("/revert_decision/"+nodeId, (e, r) => {
			if (callback !== undefined) {
				callback(e, r);
			}
		}, "POST");
	},

	getStabilityData(nodeId, behaviour, callback) {
		return this._backendRequest("/get_stability_data/"+nodeId+"/"+behaviour, (e, r) => {
			if (callback !== undefined) {
				callback(e, r);
			}
		}, "GET");
	},

	// Send a ping request. If interval is set, the ping will be repeated
	// until connection is closed. (Callback is called only once)
	ping(keepAlive = false, interval = 2000, callback = undefined) {
		// if this is a keepAlive ping, cancel any previous pings...
		if (keepAlive && this._pingRepeatToken !== undefined) {
			clearTimeout(this._pingRepeatToken);
			this._pingRepeatToken = undefined;
		}		
		this._backendRequest("/ping", (error, response) => {
			this._connected = error === undefined;
			let status = "disconnected";
			if (this._connected) {
				status = "connected";
			}
			//console.log("...ping..."+status+"...");
			if (typeof UI !== 'undefined') {
				UI.updateComputeEngineStatus(status, response);
			}			
			// Schedule a ping for later if requested.
			if (keepAlive && error === undefined) {
				this._pingRepeatToken = setTimeout(() => { this.ping(true, interval); }, interval);
			}
			if (callback !== undefined) {
				callback(error, response);
			}			
		});
	},

	// Build and return an asynchronous request with given parameters.
	_backendRequest(url, callback = undefined, method = 'GET', postData = undefined) {
		console.log(url);
        var req = new XMLHttpRequest();

        /*req.onload = function() {
        	if (callback !== undefined) {
        		let response = JSON.parse(req.response);
        		if (response.status) {
        			callback(undefined, response.result);
        		} else {	// server returned an error
        			callback(response.message, undefined);
        		}
        	}        	
        }

        req.onerror = function(e) {
        	if (callback !== undefined) {
				callback("Connection error", undefined);
        	}   
        }

        req.onabort = function() {
        	console.log("abort: ", req);
        }    

        req.open(method, this._address + url);
    	if (method == "POST" && postData !== undefined) {
    		req.send(postData);
    	} else {
    		req.send();
    	}*/

    	return req;
    },
}