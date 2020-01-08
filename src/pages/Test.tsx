import React from "react";
import { Text, View, Button, useEventHandler } from "@nodegui/react-nodegui";
import { useHistory } from "react-router";
import { QPushButtonSignals } from "@nodegui/nodegui";

export default function Test() {
  const history = useHistory();
  console.log('history:1', history);
  const handler = useEventHandler(
    { clicked: () => history.goBack() },
    []
  );
  
  return (
    <View
      style={`
        height: '100%'; 
        align-items: 'center';
        justify-content: 'center';
    `}
    >
      <Text>Test</Text>
      <Text>You are now looking at the Test Page</Text>
      <Button on={handler} text={`Go to About`}></Button>
    </View>
  );
}