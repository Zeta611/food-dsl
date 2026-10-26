module Main exposing (main)

import Browser
import Draggable
import Html exposing (Html, aside, div, h2, input, label, node, span, text)
import Html.Attributes exposing (class, property, step, style, type_, value)
import Html.Events exposing (onInput)
import Json.Encode as E


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }



-- MODEL


type alias Model =
    { params : Params
    , panelState : PanelState
    }


type alias Params =
    { radius : Float
    , tube : Float
    }


type alias PanelState =
    { pos : ( Float, Float )
    , drag : Draggable.State ()
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { params =
            { radius = 10, tube = 3 }
      , panelState =
            { pos = ( 0, 0 )
            , drag = Draggable.init
            }
      }
    , Cmd.none
    )



-- UPDATE


type Msg
    = ShapeSetRadius Float
    | ShapeSetTube Float
    | PanelOnDragBy Draggable.Delta
    | PanelDragMsg (Draggable.Msg ())


update : Msg -> Model -> ( Model, Cmd Msg )
update msg ({ params, panelState } as model) =
    case msg of
        ShapeSetRadius r ->
            ( { model | params = { params | radius = r } }, Cmd.none )

        ShapeSetTube t ->
            ( { model | params = { params | tube = t } }, Cmd.none )

        PanelOnDragBy ( dx, dy ) ->
            let
                ( x, y ) =
                    panelState.pos
            in
            ( { model
                | panelState =
                    { panelState
                        | pos = ( x + dx, y + dy )
                    }
              }
            , Cmd.none
            )

        PanelDragMsg dragMsg ->
            Tuple.mapFirst
                (\x -> { model | panelState = x })
                (Draggable.update dragConfig dragMsg panelState)


dragConfig : Draggable.Config () Msg
dragConfig =
    Draggable.basicConfig PanelOnDragBy



-- VIEW


view : Model -> Html Msg
view { params, panelState } =
    let
        ( x, y ) =
            panelState.pos

        panelTrans =
            "translate(" ++ String.fromFloat x ++ "px," ++ String.fromFloat y ++ "px)"

        dragHandle =
            Draggable.mouseTrigger () PanelDragMsg :: Draggable.touchTriggers () PanelDragMsg
    in
    div [ class "relative h-screen w-screen overflow-hidden bg-base-200" ]
        [ node "render-preview"
            [ class "absolute inset-0 block"
            , property "params" (encodeParams params)
            ]
            []
        , aside
            [ class
                "card card-sm absolute top-4 right-4 w-80 bg-base-100 border-base-300 border-2 text-base-content shadow-lg select-none"
            , style "transform" panelTrans
            ]
            [ div [ class "card-body gap-2" ]
                [ h2 (class "card-title justify-between cursor-move" :: dragHandle)
                    [ text "Configuration"
                    , span [ class "icon-[hugeicons--grip-vertical]" ] []
                    ]
                , slider "Radius"
                    { min = 2, max = 20, step = 0.5 }
                    params.radius
                    ShapeSetRadius
                , slider "Tube"
                    { min = 0.5, max = 8, step = 0.1 }
                    params.tube
                    ShapeSetTube
                ]
            ]
        ]


slider :
    String
    -> { min : Float, max : Float, step : Float }
    -> Float
    -> (Float -> Msg)
    -> Html Msg
slider name range current toMsg =
    label [ class "flex flex-col gap-1" ]
        [ span [ class "flex justify-between text-sm" ]
            [ text name
            , span [ class "text-base-content/70" ] [ text (String.fromFloat current) ]
            ]
        , input
            [ type_ "range"
            , class "range range-xs"
            , Html.Attributes.min (String.fromFloat range.min)
            , Html.Attributes.max (String.fromFloat range.max)
            , step (String.fromFloat range.step)
            , value (String.fromFloat current)
            , onInput (String.toFloat >> Maybe.withDefault current >> toMsg)
            ]
            []
        ]


encodeParams : Params -> E.Value
encodeParams { radius, tube } =
    E.object [ ( "radius", E.float radius ), ( "tube", E.float tube ) ]



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Draggable.subscriptions PanelDragMsg model.panelState.drag
