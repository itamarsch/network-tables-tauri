import { invoke } from "@tauri-apps/api/tauri";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

const NETWORK_TABLES_TCP_PORT = 5810;
/**
  * This function Connects to the robot by invoking the rust function "start_client"
  * @param {string} ip - Ip of robot to connect to. for example: "10.16.90.2"
*/
export const connectRobot = async (ip: string) =>
  await invoke("start_client", { ip: `${ip}:${NETWORK_TABLES_TCP_PORT}` })
    .then(() => console.log("Connected to RoboRIO"))
    .catch(console.error);

interface NetworkTableEvent<T> {
  readonly data: T;
  readonly timestamp: number;
}

/** 
  * This hook used for getting the state from the network tables and rerendering when new state comes.
  * @param  topic - The string for the topic name. for example: /Foo/Bar
  * @param  defaultValue - The default value for the state.
  * @returns  The value of the current state that came from the network tables entry (or is the default)
  * @example
  * const value = useReadNetworkTable("/Foo/Bar", 5)
  */
export const useReadNetworkTable = <T extends NetworkTableValue>(topic: string, defaultValue: T) => {
  const [value, setValue] = useState(defaultValue);

  useEffect(() => {
    // Not awaiting because this can complete asynchronously
    invoke("subscribe", { topic });

    const unlisten = listen<NetworkTableEvent<T>>(topic, (event) =>
      setValue(event.payload.data)
    );

    return () => {
      // Telling backend we don't want to head about this topic anymore
      invoke("unsubscribe", { topic });

      // Stop listening to this IPC channel
      unlisten.then((f: UnlistenFn) => f());
    };
  }, []);

  return value;
};

type NetworkTableValue = boolean | string | number;
type SetNetworkTableValue<T extends NetworkTableValue> = (value: T) => void;

/** This function is used for writing value to the network tables
  * @param topic - The string for the topic name. for example: /Foo/Bar
  * @returns A function for setting the value inside the network tables
  */
export const useWriteNetworkTable =
  <T extends NetworkTableValue>(topic: string): SetNetworkTableValue<T> =>
    (value: T): void => {
      invoke("write", {
        topic,
        value,
      }).catch(console.error);
    };

/** 
  * This hook used for getting the state from the network tables and rerendering when new state comes.
  * And this hook is used for writing values to the network tables
  * @param topic - The string for the topic name. for example: /Foo/Bar
  * @param defaultValue - The default value for the state.
  * @returns An array of two values, The first element of the array is the value of the current state of the entry, And the value of the second element is a function to set the value in the network tables
  * @example
  * const [value, setValue] = useReadWriteNetworkTable("/Foo/Bar", "Hello")
  * setValue("World");'
  * return (<div> {value} </div>);
  */
export const useReadWriteNetworkTable = <T extends NetworkTableValue>(
  topic: string,
  defaultValue: T
): [T, (value: T) => void] => {
  const value: T = useReadNetworkTable(topic, defaultValue);
  const setValue: (value: T) => void = useWriteNetworkTable(topic);
  return [value, setValue];
};


/**
  * This hook is used for updating the ui if the roborio is connected
  * @returns whether the robot is connected 
  */
export const useConnectedRoboRIO = (): boolean => {
  const [value, setValue] = useState(false);

  useEffect(() => {
    // Not awaiting because this can complete asynchronously
    const unlisten = listen("Connect-client", (event) =>
      setValue(event.payload as boolean)
    );

    return () => {
      unlisten.then((f: UnlistenFn) => f());
    };
  }, []);

  return value;
};
