package main

import (
    "fmt"
    "github.com/wasmerio/wasmer-go/wasmer"
    "os"
)

func main() {
    // Read the WebAssembly file
    wasmBytes, err := os.ReadFile("/tmp/rust_wasm.wasm")
    if err != nil {
        panic(fmt.Sprintf("Failed to read the wasm file: %v", err))
    }

    // Create a new WebAssembly Instance
    engine := wasmer.NewEngine()
    store := wasmer.NewStore(engine)

    // Compile the module
    module, err := wasmer.NewModule(store, wasmBytes)
    if err != nil {
        panic(fmt.Sprintf("Failed to compile module: %v", err))
    }

    // Instantiate the module
    // importObject := wasmer.NewImportObject()
    // instance, err := wasmer.NewInstance(module, importObject)
    // if err != nil {
    //     panic(fmt.Sprintf("Failed to instantiate the module: %v", err))
    // }

    wasiEnv, _ := wasmer.NewWasiStateBuilder("wasi-program").
    // Choose according to your actual situation
    // Argument("--foo").
    // Environment("ABC", "DEF").
    // MapDirectory("./", ".").
    Finalize()
    importObject, err := wasiEnv.GenerateImportObject(store, module)
    check(err)
    instance, err := wasmer.NewInstance(module, importObject)
	check(err)

    // Create memory for the instance
    limits, _ := wasmer.NewLimits(1, 100)
    memory := wasmer.NewMemory(
        store,
        // wasmer.NewMemoryType(1, true, 10),
        wasmer.NewMemoryType(limits),
    )

    // define host functions
    // Create the host function
    // Create the log_message host function
    logFn := wasmer.NewFunction(
        store,
        wasmer.NewFunctionType(
            wasmer.NewValueTypes(wasmer.I32, wasmer.I32),
            wasmer.NewValueTypes(wasmer.I32),
        ),
        func(args []wasmer.Value) ([]wasmer.Value, error) {
            ptr := args[0].I32()
            length := args[1].I32()
            
            LogMessage(memory, ptr, length)
            
            return []wasmer.Value{wasmer.NewI32(1)}, nil
        },
    )

     // Register additional host functions in the import object
     importObject.Register(
        // "env",
        "",
        map[string]wasmer.IntoExtern{
            "log_message": logFn,
            "memory": memory,
        },
    )


    // Get the exported functions
    onStart, err := instance.Exports.GetFunction("on_start")
    if err != nil {
        panic(fmt.Sprintf("Failed to get onStart function: %v", err))
    }

    onStop, err := instance.Exports.GetFunction("on_stop")
    if err != nil {
        panic(fmt.Sprintf("Failed to get onStop function: %v", err))
    }

    // Test data
    // name := "TestProcess"
    // nameBytes := []byte(name)
    // params := []string{"param1", "param2"}
    // paramsBytes := []byte(fmt.Sprintf("%v", params))

    // Call onStart
    // startResult, err := onStart(nameBytes, len(nameBytes), paramsBytes, len(paramsBytes))
    startResult, err := onStart()
    if err != nil {
        panic(fmt.Sprintf("Failed to call onStart: %v", err))
    }
    fmt.Printf("onStart result: %v\n", startResult)

    // Call onStop
    // stopResult, err := onStop(nameBytes, len(nameBytes), paramsBytes, len(paramsBytes))
    stopResult, err := onStop()
    if err != nil {
        panic(fmt.Sprintf("Failed to call onStop: %v", err))
    }
    fmt.Printf("onStop result: %v\n", stopResult)
}

// LogMessage reads a string from WebAssembly memory and logs it
func LogMessage(memory *wasmer.Memory, ptr int32, length int32) {
    // Get the memory buffer
    data := memory.Data()

    // Convert the memory segment to a string
    message := string(data[ptr : ptr+length])
    fmt.Printf("WASM Log: %s\n", message)
}

func check(e error) {
	if e != nil {
		panic(e)
	}
}
